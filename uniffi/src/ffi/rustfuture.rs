use crate::{call_with_output, FfiConverter, RustCallStatus};

use super::FfiDefault;
use std::{
    ffi::c_void,
    future::Future,
    mem::{self, ManuallyDrop},
    pin::Pin,
    ptr::NonNull,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// `RustFuture` represents a `Future` that can be sent over FFI safely-ish.
///
/// The `RustFuture` is parameterized by `T` which implements `FfiConverter`.
/// Thus, the `Future` outputs of value of kind `FfiConverter::RustType`.
/// The `poll` method maps this `FfiConverter::RustType` to
/// `FfiConverter::FfiType` when the inner `Future` is ready.
///
/// This type is not instantiated directly, but via the procedural macros,
/// e.g. `#[uniffi::export]`. A `RustFuture` is created, boxed, and then is
/// manipulated by “assistant” functions, resp. `uniffi_rustfuture_poll` and
/// `uniffi_rustfuture_drop`. Because the `RustFuture` type contains a generic
/// parameter `T`, the procedural macros will do a monomorphisation phase so that
/// all the API has all their types known.
///
/// # The big picture
///
/// This section will explain how the entire workflow works.
///
/// Let's consider the following Rust function:
///
/// ```rs,ignore
/// #[uniffi::export]
/// async fn hello() -> bool {
///     true
/// }
/// ```
///
/// By the way, in Rust, this `async fn` syntax is strictly equivalent to:
///
/// ```rs,ignore
/// #[uniffi::export]
/// fn hello() -> impl Future<Output = bool> { … }
/// ```
///
/// Once this is understood, it becomes obvious that an `async` function
/// returns a `Future`.
///
/// This function will be not be modified, but new C-ABI/FFI compatible
/// functions will be created, as such:
///
/// ```rs,ignore
/// /// The `hello` function, as seen from the outside. It returns a “`Future`”, or
/// /// more precisely, a `RustFuture` that wraps the returned future.
/// #[no_mangle]
/// pub extern "C" fn _uniffi_hello() -> Option<Box<::uniffi::RustFuture<bool>>> {
///     ::uniffi::call_with_output(::uniffi::RustFuture::new(async move {
///         hello().await
///     }))
/// }
/// ```
///
/// The second function generated is the _poll function_:
///
/// ```rs,ignore
/// /// The function that polls the `RustFuture` returned by `_uniffi_hello`.
/// #[no_mangle]
/// pub extern "C" fn _uniffi_hello_poll(
///     future: Option<&mut ::uniffi::RustFuture<bool>>,
///     waker: Option<NonNull<::uniffi::RustFutureForeignWakerFunction>>,
///     waker_environment: *const ::uniffi::RustFutureForeignWakerEnvironment,
///     polled_result: &mut <bool as ::uniffi::FfiConverter>::FfiType,
///     call_status:: &mut ::uniffi::RustCallStatus,
/// ) -> bool {
///     ::uniffi::ffi::uniffi_rustfuture_poll(future, waker, waker_environment, polled_result, call_status)
/// }
/// ```
///
/// Let's analyse this function because it's an important one.
///
/// First off, this _poll FFI function_ forwards everything to
/// `uniffi_rustfuture_poll`. This later is generic, while the former has been
/// monomorphized by the procedural macros.
///
/// Second, it receives the `RustFuture` as an `Option<&mut RustFuture<_>>`. It
/// doesn't take ownership of the `RustFuture`! It borrows it (mutably). It's
/// wrapped inside an `Option` to check whether it's a null pointer or not.
///
/// Third, it receives a _waker_ as a pair of a _function pointer_ plus its
/// _environment_ if any: a null pointer is purposely allowed here. This waker
/// function lives on the foreign language land. We will come back to it in a
/// second.
///
/// Fourth, it receives an in-out `polled_result` argument, that is filled with the
/// polled result if the future is ready (this part is subject to change).
///
/// Finally, the clasical `call_status`, which is part of the calling API of `uniffi`.
///
/// So, everytime this function is called, it polls the `RustFuture` after
/// having reconstitued a valid [`Waker`](https://doc.rust-lang.org/std/task/ struct.Waker.html)
/// for it. As said earlier, we will come back to it.
///
/// The last function generated automatically is the _drop function_
///
/// ```rs,ignore
/// #[no_mangle]
/// pub extern "C" fn _uniffi_hello_drop(
///     future: Option<Box<::uniffi::RustFuture<bool>>>,
///     call_status: &mut ::uniffi::RustCallStatus
/// ) {
///     ::uniffi::ffi::uniffi_rustfuture_drop(future, call_status)
/// }
/// ```
///
/// First off, this _drop function_ is responsible to drop the `RustFuture`. It's
/// clear by looking at its signature: It receives a `Option<Box<RustFuture<_>>>`,
/// i.e. it takes ownership of the `RustFuture` via `Box`!
///
/// Similarly to the _poll function_, it forwards everything to
/// `uniffi_rustfuture_drop`, which the generic version of the monomorphized _drop
/// function_.
///
/// ## How `Future` works exactly?
///
/// A `Future` in Rust does nothing. When calling an async function, it just
/// returns a `Future` but nothing has happened yet. To start the computation,
/// the future must be polled. It returns `Poll::Ready(r)` if the result is ready,
/// `Poll::Pending` otherwise. `Poll::Pending` basically means:
///
/// > Please, try to poll me later, maybe the result will be ready!
///
/// This model is very different than what other languages do, but it can actually
/// be mangled quite easily, hopefully for us!
///
/// But… wait a minute… who is responsible to poll the `Future` if a `Future` does
/// nothing? Well, it's _the executor_ (sometimes also called _the runtime_ but not
/// here, we stay with _the executor_). The executor is responsible _to drive_ the
/// `Future`, that's where they are are polled.
///
/// But… wait another minute… how the executor knows when to poll a `Future`? Does
/// it poll them randomly in a forever loop? Well, it depends of the executor and
/// let's hope it will never happen! But normally, when `Future::poll` is called, a
/// `Context` argument is passed to it. It contains a `Waker`. The `Waker` is built
/// on top of a `RawWaker` which implements whatever is necessary. Usually, a waker
/// will signal the executor to poll a particular `Future`. A `Future` will clone
/// or pass-by-ref the waker to somewhere, as a callback, a completion, a function,
/// or anything, to the system that is responsible to notify when a task is
/// completed. So, to recap, the waker is _not_ responsible to wake the `Future`,
/// it _is_ responsible to _signal_ the executor that a particular `Future` must be
/// polled again.
///
/// ## Awaken from the foreign language land
///
/// The _poll function_ receives a waker function pointer, along with a waker
/// environment. We said that the waker function lives in the foreign language
/// land. That's really important. It cannot live inside Rust because Rust isn't
/// aware of which foreign language it is run from. It is UniFFI's job to write a
/// proper waker function, that will use the native foreign language's executor, or
/// some common library (e.g. `asyncio` in Python).
///
/// ## The workflow
///
/// 1. The foreign language starts by calling the regular FFI function
///    `_uniffi_hello`. It gets a `Box<RustFuture<_>>`.
///
/// 2. The foreign language polls the future by using the `_uniffi_hello_poll`
///    function. It passes a function pointer to the waker function, implemented
///    inside the foreign language, along with its environment if any.
///
///   - Either the future is ready and computes a value, in this case the _poll
///     function_ will lift the value and will drop the future with the _drop function_
///     (`_uniffi_hello_drop`),
///
///   - Either the future is pending (not ready), and is responsible to register
///     the waker (it's pretty classical in Rust, that's the canonical way to do).
///
/// 3. When the waker is called, it calls the _poll function_, so we basically jump
///    to point 2 of this list.
///
/// There is an important subtlety though. Imagine the following Rust code:
///
/// ```rs,ignore
/// let mut shared_state: MutexGuard<_> = a_shared_state.lock().unwrap();
///
/// if let Some(waker) = shared_state.waker.take() {
///     waker.wake();
/// }
/// ```
///
/// This code will call the waker. That's nice and all. It means that the function
/// has not returned yet and it's calling the waker function. The waker function
/// will call the _poll function_ on the Rust side, which… may use the same lock
/// (`a_shared_state`) as in this code above. Rust is not responsible of that, kind
/// of. Rust **must ignore how the executor works**, all `Future`s are executor-
/// agnostic by design. To avoid creating problem, the waker must “cut” the flow,
/// so that Rust code can continue to run as expected, and then the _poll function_
/// must be called. Put it differently, the waker function must call the _poll
/// function_ _as soon as possible_, not _immediately_. It actually makes sense:
/// The waker must signal the executor to poll a specific `Future` as soon as
/// possible, it must not be done immediately. The implementation of the executor
/// must be very careful of that.
///
/// With a diagram (because this comment would look so much clever with a diagram),
/// it looks like this:
///
/// ```
///           ┌────────────────────┐
///           │                    │
///           │   Calling hello    │
///           │                    │
///           └─────────┬──────────┘
///                     │
///                     ▼       fn waker ◄──┐
///     ┌────────────────────────────────┐  │
///     │                                │  │
///     │  Ask the executor to do this   │  │
///     │  as soon as possible           │  │
///     │                                │  │
///     │  ┌──────────────────────────┐  │  │
///     │  │                          │  │  │
///     │  │  Polling the RustFuture  │  │  │
///     │  │   Pass pointer to waker ─┼──┼──┘
///     │  │                          │  │
///     │  └────────────┬─────────────┘  │
///     │               │                │
///     └───────────────┼────────────────┘
///                     │
///                     ▼
///         ┌──── The future is ─────┐
///         │                        │
///       Ready                   Pending
///         │                        │
///         ▼                        ▼
/// ┌───────────────┐     ┌──────────────────────┐
/// │  Lift result  │     │      Nothing         │
/// │   Have fun    │     │ Let's wait the waker │
/// └───────────────┘     │    to be called      │
///                       └──────────────────────┘
/// ```
#[repr(transparent)]
pub struct RustFuture<T>(Pin<Box<dyn Future<Output = <T as FfiConverter>::RustType> + 'static>>)
where
    T: FfiConverter;

impl<T> RustFuture<T>
where
    T: FfiConverter,
{
    /// Create a new `RustFuture` from a regular `Future` that outputs a value
    /// of kind `FfiConverter::RustType`.
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = <T as FfiConverter>::RustType> + 'static,
    {
        Self(Box::pin(future))
    }

    /// Poll the future. It basically maps
    /// `<T as FfiConverter>::RustType` to `<T as FfiConverter>::FfiType` when
    /// the inner future is ready.
    ///
    /// There is one subtlety compared to `Future::poll` though: the
    /// `foreign_waker_pointer`. We want the `RustFuture` **to be driven by
    /// the foreign language's runtime/executor**. Hence the possibility for
    /// the foreign language to provide its own waker function: Rust can signal
    /// something has happened within the future and should be polled again.
    fn poll(
        &mut self,
        foreign_waker: *const RustFutureForeignWakerFunction,
        foreign_waker_environment: *const RustFutureForeignWakerEnvironment,
    ) -> Poll<<T as FfiConverter>::FfiType> {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(Arc::new(RustFutureForeignWaker {
                    waker: foreign_waker,
                    waker_environment: foreign_waker_environment,
                })) as *const (),
                &RustTaskWakerBuilder::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        Pin::new(&mut self.0)
            .poll(&mut context)
            .map(<T as FfiConverter>::lower)
    }
}

impl<T> FfiDefault for Option<Box<RustFuture<T>>>
where
    T: FfiConverter,
{
    fn ffi_default() -> Self {
        None
    }
}

impl<T> FfiDefault for Poll<T> {
    /// The default value for `Poll<T>` is `Poll::Pending`.
    fn ffi_default() -> Self {
        Self::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_future_size() {
        let pointer_size = mem::size_of::<*const u8>();
        let rust_future_size = pointer_size * 2;

        assert_eq!(mem::size_of::<RustFuture::<bool>>(), rust_future_size,);
        assert_eq!(mem::size_of::<RustFuture::<u64>>(), rust_future_size,);
    }
}

/// Type alias to a function with a C ABI. It defines the shape of
/// the foreign language's waker which is called by the `RustFuture` (more
/// precisely, by its inner future) to signal the foreign language that
/// something has happened. See [`RustFuture::poll`] to learn more.
pub type RustFutureForeignWakerFunction =
    unsafe extern "C" fn(*const RustFutureForeignWakerEnvironment);
pub type RustFutureForeignWakerEnvironment = c_void;

#[derive(Debug)]
struct RustFutureForeignWaker {
    waker: *const RustFutureForeignWakerFunction,
    waker_environment: *const RustFutureForeignWakerEnvironment,
}

/// Zero-sized type to create the VTable for the `RawWaker`.
struct RustTaskWakerBuilder;

impl RustTaskWakerBuilder {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    /// This function will be called when the `RawWaker` gets cloned, e.g. when
    /// the `Waker` in which the `RawWaker` is stored gets cloned.
    unsafe fn clone_waker(foreign_waker: *const ()) -> RawWaker {
        let waker = foreign_waker as *const RustFutureForeignWaker;
        Arc::increment_strong_count(waker);

        RawWaker::new(foreign_waker, &Self::VTABLE)
    }

    /// This function will be called when `wake` is called on the `Waker`. It
    /// must wake up the task associated with this `RawWaker`.
    unsafe fn wake(foreign_waker: *const ()) {
        let waker = foreign_waker as *const RustFutureForeignWaker;
        let waker = Arc::from_raw(waker);

        let func = mem::transmute::<_, RustFutureForeignWakerFunction>(waker.waker);
        func(waker.waker_environment);
    }

    /// This function will be called when `wake_by_ref` is called on the
    /// `Waker`. It must wake up the task associated with this `RawWaker`.
    unsafe fn wake_by_ref(foreign_waker: *const ()) {
        let waker = foreign_waker as *const RustFutureForeignWaker;
        let waker = ManuallyDrop::new(Arc::from_raw(waker));

        let func = mem::transmute::<_, RustFutureForeignWakerFunction>(waker.waker);
        func(waker.waker_environment);
    }

    /// This function gets called when a `RawWaker` gets dropped.
    unsafe fn drop_waker(foreign_waker: *const ()) {
        let waker = foreign_waker as *const RustFutureForeignWaker;
        drop(Arc::from_raw(waker));
    }
}

/// Poll a `RustFuture`. If the `RustFuture` is ready, the function returns
// `true` and puts the result inside `polled_result`, otherwise it returns `false`
// and _doesn't change_ the value inside `polled_result`.
///
/// # Panics
///
/// The function panics if `future` or `waker` is a NULL pointer.
#[doc(hidden)]
pub fn uniffi_rustfuture_poll<T>(
    future: Option<&mut RustFuture<T>>,
    waker: Option<NonNull<RustFutureForeignWakerFunction>>,
    waker_environment: *const RustFutureForeignWakerEnvironment,
    polled_result: &mut <T as FfiConverter>::FfiType,
    call_status: &mut RustCallStatus,
) -> bool
where
    T: FfiConverter,
{
    let future = future.expect("`future` is a null pointer");
    let waker = waker.expect("`waker` is a null pointer");

    let future_mutex = Mutex::new(future);

    match call_with_output(call_status, || {
        future_mutex
            .lock()
            .unwrap()
            .poll(waker.as_ptr(), waker_environment)
    }) {
        Poll::Ready(result) => {
            *polled_result = result;

            true
        }

        Poll::Pending => false,
    }
}

/// Drop a `RustFuture`.
///
/// Because this function takes ownership of `future` (by `Box`ing it), the
/// future will be dropped naturally by the compiler at the end of the function
/// scope.
#[doc(hidden)]
pub fn uniffi_rustfuture_drop<T>(
    _future: Option<Box<RustFuture<T>>>,
    _call_status: &mut RustCallStatus,
) where
    T: FfiConverter,
{
}
