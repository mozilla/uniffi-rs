//! [`RustFuture`] represents a [`Future`] that can be sent over FFI safely-ish.
//!
//! The [`RustFuture`] type holds an inner `Future<Output = Result<E, T>>`, and
//! thus is parameterized by `T` and `E`. On the `RustFuture` type itself, there
//! is no constraint over those generic types (constraints are present in the
//! [`uniffi_rustfuture_poll`] function, where `T: FfiReturn`, see later
//! to learn more). Every function or method that returns a `Future` must
//! transform the result into a `Result`.
//!
//! This type may not be instantiated directly, but _via_ the procedural macros,
//! such as `#[uniffi::export]`. A `RustFuture` is created, boxed, and then
//! manipulated by (hidden) helper functions, resp. [`uniffi_rustfuture_poll`]
//! and [`uniffi_rustfuture_drop`]. Because the `RustFuture` type contains a
//! generic parameters `T` and `E`, the procedural macros will do a
//! monomorphisation phase so that all the API has all their types statically
//! known.
//!
//! # The big picture
//!
//! This section will explain how the entire workflow works.
//!
//! Let's consider the following Rust function:
//!
//! ```rust,ignore
//! #[uniffi::export]
//! async fn hello() -> bool {
//!     true
//! }
//! ```
//!
//! In Rust, this `async fn` syntax is strictly equivalent to:
//!
//! ```rust,ignore
//! #[uniffi::export]
//! fn hello() -> impl Future<Output = bool> { /* … */ }
//! ```
//!
//! Once this is understood, it becomes obvious that an `async` function
//! returns a `Future`.
//!
//! This function will not be modified, but new functions with a C ABI will be
//! created, as such:
//!
//! ```rust,ignore
//! /// The `hello` function, as seen from the outside. It returns a `Future`, or
//! /// more precisely, a `RustFuture` that wraps the returned future.
//! #[no_mangle]
//! pub extern "C" fn _uniffi_hello(
//!     call_status: &mut ::uniffi::RustCallStatus
//! ) -> Option<Box<::uniffi::RustFuture<bool, ::std::convert::Infallible>>> {
//!     ::uniffi::call_with_output(call_status, || {
//!         Some(Box::new(::uniffi::RustFuture::new(async move {
//!             Ok(hello().await)
//!         })))
//!     })
//! }
//! ```
//!
//! The second generated function is the _poll function_:
//!
//! ```rust,ignore
//! /// The function to poll the `RustFuture` returned by `_uniffi_hello`.
//! #[no_mangle]
//! pub extern "C" fn _uniffi_hello_poll(
//!     future: Option<&mut ::uniffi::RustFuture<bool, ::std::convert::Infallible>>,
//!     waker: Option<NonNull<::uniffi::RustFutureForeignWakerFunction>>,
//!     waker_environment: *const ::uniffi::RustFutureForeignWakerEnvironment,
//!     polled_result: &mut <bool as ::uniffi::FfiReturn>::FfiType,
//!     call_status:: &mut ::uniffi::RustCallStatus,
//! ) -> bool {
//!     ::uniffi::ffi::uniffi_rustfuture_poll(future, waker, waker_environment, polled_result, call_status)
//! }
//! ```
//!
//! Let's analyse this function because it's an important one:
//!
//! * First off, this _poll FFI function_ forwards everything to
//!   [`uniffi_rustfuture_poll`]. The latter is generic, while the former has been
//!   monomorphised by the procedural macro.
//!
//! * Second, it receives the `RustFuture` as an `Option<&mut RustFuture<_>>`. It
//!   doesn't take ownership of the `RustFuture`! It borrows it (mutably). It's
//!   wrapped inside an `Option` to check whether it's a null pointer or not.
//!
//! * Third, it receives a _waker_ as a pair of a _function pointer_ plus its
//!   _environment_, if any; a null pointer is purposely allowed for the environment.
//!   This waker function lives on the foreign language side. We will come back
//!   to it in a second.
//!
//! * Fourth, it receives an in-out `polled_result` argument, that is filled with the
//!   polled result if the future is ready.
//!
//! * Finally, the classical `call_status`, which is part of the calling API of `uniffi`.
//!
//! So, everytime this function is called, it polls the `RustFuture` after
//! having reconstituted a valid [`Waker`] for it. As said earlier, we will come
//! back to it.
//!
//! The last generated function is the _drop function_:
//!
//! ```rust,ignore
//! #[no_mangle]
//! pub extern "C" fn _uniffi_hello_drop(
//!     future: Option<Box<::uniffi::RustFuture<bool, ::std::convert::Infallible>>>,
//!     call_status: &mut ::uniffi::RustCallStatus
//! ) {
//!     ::uniffi::ffi::uniffi_rustfuture_drop(future, call_status)
//! }
//! ```
//!
//! First off, this _drop function_ is responsible to drop the `RustFuture`. It's
//! clear by looking at its signature: It receives an `Option<Box<RustFuture<_>>>`,
//! i.e. it takes ownership of the `RustFuture` _via_ `Box`!
//!
//! Similarly to the _poll function_, it forwards everything to
//! [`uniffi_rustfuture_drop`], which is the generic version of the monomorphised _drop
//! function_.
//!
//! ## How does `Future` work exactly?
//!
//! A [`Future`] in Rust does nothing. When calling an async function, it just
//! returns a `Future` but nothing has happened yet. To start the computation,
//! the future must be polled. It returns [`Poll::Ready(r)`][`Poll::Ready`] if
//! the result is ready, [`Poll::Pending`] otherwise. `Poll::Pending` basically
//! means:
//!
//! > Please, try to poll me later, maybe the result will be ready!
//!
//! This model is very different than what other languages do, but it can actually
//! be translated quite easily, fortunately for us!
//!
//! But… wait a minute… who is responsible to poll the `Future` if a `Future` does
//! nothing? Well, it's _the executor_. The executor is responsible _to drive_ the
//! `Future`: that's where they are polled.
//!
//! But… wait another minute… how does the executor know when to poll a [`Future`]?
//! Does it poll them randomly in an endless loop? Well, no, actually it depends
//! on the executor! A well-designed `Future` and executor work as follows.
//! Normally, when [`Future::poll`] is called, a [`Context`] argument is
//! passed to it. It contains a [`Waker`]. The [`Waker`] is built on top of a
//! [`RawWaker`] which implements whatever is necessary. Usually, a waker will
//! signal the executor to poll a particular `Future`. A `Future` will clone
//! or pass-by-ref the waker to somewhere, as a callback, a completion, a
//! function, or anything, to the system that is responsible to notify when a
//! task is completed. So, to recap, the waker is _not_ responsible for waking the
//! `Future`, it _is_ responsible for _signaling_ the executor that a particular
//! `Future` should be polled again. That's why the documentation of
//! [`Poll::Pending`] specifies:
//!
//! > When a function returns `Pending`, the function must also ensure that the
//! > current task is scheduled to be awoken when progress can be made.
//!
//! “awakening” is done by using the `Waker`.
//!
//! ## Awaken from the foreign language land
//!
//! Our _poll function_ receives a waker function pointer, along with a waker
//! environment. We said that the waker function lives on the foreign language
//! side. That's really important. It cannot live inside Rust because Rust
//! isn't aware of which foreign language it is run from, and thus doesn't know
//! which executor is used. It is UniFFI's job to write a proper foreign waker
//! function that will use the native foreign language's executor provided
//! by the foreign language itself (e.g. `Task` in Swift) or by some common
//! libraries (e.g. `asyncio` in Python), to ask to poll the future again.
//!
//! ## The workflow
//!
//! 1. The foreign language starts by calling the regular FFI function
//!    `_uniffi_hello`. It gets an `Option<Box<RustFuture<_>>>`.
//!
//! 2. The foreign language polls the future by using the `_uniffi_hello_poll`
//!    function. It passes a function pointer to the waker function, implemented
//!    inside the foreign language, along with its environment if any.
//!
//!    - Either the future is ready and computes a value, in this case the _poll
//!      function_ will lift the value and will drop the future with the _drop function_
//!      (`_uniffi_hello_drop`),
//!
//!    - or the future is pending (not ready), and is responsible to register
//!      the waker (as explained above).
//!
//! 3. When the waker is called, it calls the _poll function_, so we basically jump
//!    to point 2 of this list.
//!
//! There is an important subtlety though. Imagine the following Rust code:
//!
//! ```rust,ignore
//! let mut shared_state: MutexGuard<_> = a_shared_state.lock().unwrap();
//!
//! if let Some(waker) = shared_state.waker.take() {
//!     waker.wake();
//! }
//! ```
//!
//! This code will call the waker. That's nice and all. However, when the waker
//! function is called by `waker.wake()`, this code above has not returned yet.
//! And the waker function, as designed so far, will call the _poll function_
//! of the Rust `Future`  which… may use the same lock (`a_shared_state`),
//! which is not released yet: there is a dead-lock! Rust is not responsible of
//! that, kind of. Rust **must ignore how the executor works**, all `Future`s
//! are executor-agnostic by design. To avoid creating problems, the waker
//! must “cut” the flow, so that Rust code can continue to run as expected, and
//! after that, the _poll function_ must be called.
//!
//! Put differently, the waker function must call the _poll function_ _as
//! soon as possible_, not _immediately_. It actually makes sense: The waker
//! must signal the executor to schedule a poll for a specific `Future` when
//! possible; it's not an inline operation. The implementation of the waker
//! must be very careful of that.
//!
//! With a diagram (because this comment would look so much clever with a diagram),
//! it looks like this:
//!
//! ```text
//!           ┌────────────────────┐
//!           │                    │
//!           │   Calling hello    │
//!           │                    │
//!           └─────────┬──────────┘
//!                     │
//!                     ▼       fn waker ◄──┐
//!     ┌────────────────────────────────┐  │
//!     │                                │  │
//!     │  Ask the executor to schedule  │  │
//!     │  this as soon as possible      │  │
//!     │                                │  │
//!     │  ┌──────────────────────────┐  │  │
//!     │  │                          │  │  │
//!     │  │  Polling the RustFuture  │  │  │
//!     │  │  Pass pointer to waker ──┼──┼──┘
//!     │  │                          │  │
//!     │  └────────────┬─────────────┘  │
//!     │               │                │
//!     └───────────────┼────────────────┘
//!                     │
//!                     ▼
//!         ┌──── The future is ─────┐
//!         │                        │
//!       Ready                   Pending
//!         │                        │
//!         ▼                        ▼
//! ┌───────────────┐     ┌──────────────────────┐
//! │  Lift result  │     │       Nothing        │
//! │   Have fun    │     │  Let's wait for the  │
//! └───────────────┘     │  waker to be called  │
//!                       └──────────────────────┘
//! ```
//!
//! That's all folks!
//!
//! [`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
//! [`Future::poll`]: https://doc.rust-lang.org/std/future/trait.Future.html#tymethod.poll
//! [`Pol::Ready`]: https://doc.rust-lang.org/std/task/enum.Poll.html#variant.Ready
//! [`Poll::Pending`]: https://doc.rust-lang.org/std/task/enum.Poll.html#variant.Pending
//! [`Context`]: https://doc.rust-lang.org/std/task/struct.Context.html
//! [`Waker`]: https://doc.rust-lang.org/std/task/struct.Waker.html
//! [`RawWaker`]: https://doc.rust-lang.org/std/task/struct.RawWaker.html

use super::FfiDefault;
use crate::{call_with_result, FfiConverter, FfiError, FfiReturn, RustBuffer, RustCallStatus};
use std::{
    ffi::c_void,
    future::Future,
    mem::ManuallyDrop,
    panic,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// `RustFuture` represents a [`Future`] that can be sent over FFI safely-ish.
///
/// See the module documentation to learn more.
#[repr(transparent)]
pub struct RustFuture<T, E>(Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>);

impl<T, E> RustFuture<T, E> {
    /// Create a new `RustFuture` from a regular `Future`.
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = Result<T, E>> + 'static,
    {
        Self(Box::pin(future))
    }

    /// Create a new `RustFuture` from a Tokio `Future`. It needs an
    /// intermediate compatibility layer to be handlded as a regular Rust
    /// `Future`.
    #[cfg(feature = "tokio")]
    pub fn new_tokio<F>(future: F) -> Self
    where
        F: Future<Output = Result<T, E>> + 'static,
    {
        Self(Box::pin(async_compat::Compat::new(future)))
    }

    /// Poll the future.
    ///
    /// There is one subtlety compared to `Future::poll` though: the
    /// `foreign_waker` and `foreign_waker_environment` variables replace the
    /// classical [`Context`]. We want the `RustFuture` **to be driven by the
    /// foreign language's executor**. Hence the possibility for the foreign
    /// language to provide its own waker function: Rust can signal something
    /// has happened within the future and should be polled again.
    ///
    /// `Poll` is mapped to [`RustFuturePoll`].
    ///
    /// [`Context`]: https://doc.rust-lang.org/std/task/struct.Context.html
    fn poll(
        &mut self,
        foreign_waker: RustFutureForeignWakerFunction,
        foreign_waker_environment: *const RustFutureForeignWakerEnvironment,
    ) -> RustFuturePoll<Result<T, E>> {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                RustFutureForeignWaker::new(foreign_waker, foreign_waker_environment)
                    .into_unit_ptr(),
                &RustFutureForeignRawWaker::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        self.0.as_mut().poll(&mut context).into()
    }
}

impl<T, E> FfiDefault for Option<Box<RustFuture<T, E>>> {
    fn ffi_default() -> Self {
        None
    }
}

/// `RustFuturePoll` is the equivalent of [`Poll`], except that it has one
/// more variant: `Throwing`, which is the ”FFI default” variant.
///
/// Why? The [`FfiDefault`] trait is used to compute a default value when an
/// error must be returned. It must be reflected inside the `RustFuturePoll`
/// type to know in which state the `RustFuture` is.
///
/// [`Poll`]: https://doc.rust-lang.org/std/task/enum.Poll.html
#[derive(Debug)]
pub enum RustFuturePoll<T> {
    /// A value `T` is ready!
    Ready(T),

    /// Naah, please try later, maybe a value will be ready?
    Pending,

    /// The default state for `FfiDefault`, indicating that the `RustFuture` is
    /// throwing an error.
    Throwing,
}

impl<T> From<Poll<T>> for RustFuturePoll<T> {
    fn from(value: Poll<T>) -> Self {
        match value {
            Poll::Ready(ready) => Self::Ready(ready),
            Poll::Pending => Self::Pending,
        }
    }
}

impl<T, E> RustFuturePoll<Result<T, E>> {
    /// Transpose a `RustFuturePoll<Result<T, E>>` to a
    /// `Result<RustFuturePoll<T>, E>`.
    fn transpose(self) -> Result<RustFuturePoll<T>, E> {
        match self {
            Self::Ready(ready) => match ready {
                Ok(ok) => Ok(RustFuturePoll::Ready(ok)),
                Err(error) => Err(error),
            },

            Self::Pending => Ok(RustFuturePoll::Pending),

            Self::Throwing => Ok(RustFuturePoll::Throwing),
        }
    }
}

impl<T> FfiDefault for RustFuturePoll<T> {
    fn ffi_default() -> Self {
        Self::Throwing
    }
}

#[cfg(test)]
mod tests_rust_future {
    use super::*;
    use std::mem;

    #[test]
    fn test_rust_future_size() {
        let pointer_size = mem::size_of::<*const u8>();
        let rust_future_size = pointer_size * 2;

        assert_eq!(mem::size_of::<RustFuture::<bool, ()>>(), rust_future_size);
        assert_eq!(mem::size_of::<RustFuture::<u64, ()>>(), rust_future_size);
    }
}

/// Type alias to a function with a C ABI. It defines the shape of the foreign
/// language's waker which is called by the [`RustFuture`] to signal the
/// foreign language that something has happened. See the module documentation
/// to learn more.
pub type RustFutureForeignWakerFunction =
    unsafe extern "C" fn(*const RustFutureForeignWakerEnvironment);

/// Type alias for the environment of a [`RustFutureForeignWakerFunction`].
/// It's an alias to C `void`, which basically means here that the environment
/// can be anything. See the module documentation to learn more.
pub type RustFutureForeignWakerEnvironment = c_void;

#[derive(Debug)]
struct RustFutureForeignWaker {
    waker: RustFutureForeignWakerFunction,
    waker_environment: *const RustFutureForeignWakerEnvironment,
}

impl RustFutureForeignWaker {
    fn new(
        waker: RustFutureForeignWakerFunction,
        waker_environment: *const RustFutureForeignWakerEnvironment,
    ) -> Self {
        Self {
            waker,
            waker_environment,
        }
    }

    fn into_unit_ptr(self) -> *const () {
        Arc::into_raw(Arc::new(self)) as *const ()
    }

    unsafe fn increment_reference_count(me: *const ()) {
        Arc::increment_strong_count(me as *const Self);
    }

    unsafe fn from_unit_ptr(me: *const ()) -> Arc<Self> {
        Arc::from_raw(me as *const Self)
    }
}

/// Zero-sized type to create the VTable
/// ([Virtual method table](https://en.wikipedia.org/wiki/Virtual_method_table))
/// for the `RawWaker`.
struct RustFutureForeignRawWaker;

impl RustFutureForeignRawWaker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    /// This function will be called when the `RawWaker` gets cloned, e.g. when
    /// the `Waker` in which the `RawWaker` is stored gets cloned.
    unsafe fn clone_waker(foreign_waker: *const ()) -> RawWaker {
        RustFutureForeignWaker::increment_reference_count(foreign_waker);

        RawWaker::new(foreign_waker, &Self::VTABLE)
    }

    /// This function will be called when `wake` is called on the `Waker`. It
    /// must wake up the task associated with this `RawWaker`.
    unsafe fn wake(foreign_waker: *const ()) {
        let waker = RustFutureForeignWaker::from_unit_ptr(foreign_waker);
        let func = waker.waker;

        func(waker.waker_environment);
    }

    /// This function will be called when `wake_by_ref` is called on the
    /// `Waker`. It must wake up the task associated with this `RawWaker`.
    unsafe fn wake_by_ref(foreign_waker: *const ()) {
        let waker = ManuallyDrop::new(RustFutureForeignWaker::from_unit_ptr(foreign_waker));
        let func = waker.waker;

        func(waker.waker_environment);
    }

    /// This function gets called when a `RawWaker` gets dropped.
    unsafe fn drop_waker(foreign_waker: *const ()) {
        drop(RustFutureForeignWaker::from_unit_ptr(foreign_waker));
    }
}

#[cfg(test)]
mod tests_raw_waker_vtable {
    use super::*;
    use std::{cell::RefCell, ptr};

    // This entire `RustFuture` stuff assumes the waker lives in the foreign
    // language, but for the sake of testing, we will fake it.
    extern "C" fn my_waker(env: *const c_void) {
        let env = ManuallyDrop::new(unsafe { Box::from_raw(env as *mut RefCell<bool>) });
        env.replace(true);

        // do something smart!
    }

    #[test]
    fn test_rust_future_foreign_waker_basic_manipulations() {
        let foreign_waker_ptr =
            RustFutureForeignWaker::new(my_waker as _, ptr::null()).into_unit_ptr();
        let foreign_waker: Arc<RustFutureForeignWaker> =
            unsafe { RustFutureForeignWaker::from_unit_ptr(foreign_waker_ptr) };

        assert_eq!(Arc::strong_count(&foreign_waker), 1);
    }

    #[test]
    fn test_clone_and_drop_waker() {
        let foreign_waker_ptr =
            RustFutureForeignWaker::new(my_waker as _, ptr::null()).into_unit_ptr();
        let foreign_waker = unsafe { RustFutureForeignWaker::from_unit_ptr(foreign_waker_ptr) };

        let _ = unsafe { RustFutureForeignRawWaker::clone_waker(foreign_waker_ptr) };
        assert_eq!(Arc::strong_count(&foreign_waker), 2);

        unsafe { RustFutureForeignRawWaker::drop_waker(foreign_waker_ptr) };
        assert_eq!(Arc::strong_count(&foreign_waker), 1);
    }

    #[test]
    fn test_wake() {
        let foreign_waker_environment_ptr = Box::into_raw(Box::new(RefCell::new(false)));
        let foreign_waker_ptr =
            RustFutureForeignWaker::new(my_waker as _, foreign_waker_environment_ptr as *const _)
                .into_unit_ptr();
        let foreign_waker = unsafe { RustFutureForeignWaker::from_unit_ptr(foreign_waker_ptr) };

        // Clone to increase the strong count, so that we can see if it's been dropped by `wake` later.
        let _ = unsafe { RustFutureForeignRawWaker::clone_waker(foreign_waker_ptr) };
        assert_eq!(Arc::strong_count(&foreign_waker), 2);

        // Let's call the waker.
        unsafe { RustFutureForeignRawWaker::wake(foreign_waker_ptr) };

        // Has it been called?
        let foreign_waker_environment = unsafe { Box::from_raw(foreign_waker_environment_ptr) };
        assert!(foreign_waker_environment.take());

        // Has the waker been dropped?
        assert_eq!(Arc::strong_count(&foreign_waker), 1);
    }

    #[test]
    fn test_wake_by_ref() {
        let foreign_waker_environment_ptr = Box::into_raw(Box::new(RefCell::new(false)));
        let foreign_waker_ptr =
            RustFutureForeignWaker::new(my_waker as _, foreign_waker_environment_ptr as *const _)
                .into_unit_ptr();
        let foreign_waker = unsafe { RustFutureForeignWaker::from_unit_ptr(foreign_waker_ptr) };

        // Clone to increase the strong count, so that we can see if it has not been dropped by `wake_by_ref` later.
        let _ = unsafe { RustFutureForeignRawWaker::clone_waker(foreign_waker_ptr) };
        assert_eq!(Arc::strong_count(&foreign_waker), 2);

        // Let's call the waker by reference.
        unsafe { RustFutureForeignRawWaker::wake_by_ref(foreign_waker_ptr) };

        // Has it been called?
        let foreign_waker_environment = unsafe { Box::from_raw(foreign_waker_environment_ptr) };
        assert!(foreign_waker_environment.take());

        // Has the waker not been dropped?
        assert_eq!(Arc::strong_count(&foreign_waker), 2);

        // Dropping manually to avoid data leak.
        unsafe { RustFutureForeignRawWaker::drop_waker(foreign_waker_ptr) };
    }
}

const READY: bool = true;
const PENDING: bool = false;

/// Poll a [`RustFuture`]. If the `RustFuture` is ready, the function returns
/// `true` and puts the result inside `polled_result`, otherwise it returns
/// `false` and _doesn't modify_ the value inside `polled_result`. A third
/// case exists: if the `RustFuture` is throwing an error, the function returns
/// `true` but doesn't modify `polled_result` either, however the value
/// of  `call_status` is changed appropriately. It is summarized inside the
/// following table:
///
/// | `RustFuture`'s state | `polled_result`         | `call_status.code` | returned value |
/// |----------------------|-------------------------|--------------------|----------------|
/// | `Ready(Ok(T))`       | is mapped to `T::lower` | `CALL_SUCCESS`     | `true`         |
/// | `Ready(Err(E))`      | is not modified         | `CALL_ERROR`       | `true`         |
/// | `Pending`            | is not modified         | `CALL_SUCCESS`     | `false`        |
///
/// Please see the module documentation to learn more.
///
/// # Panics
///
/// The function panics if `future` or `waker` is a NULL pointer.
pub fn uniffi_rustfuture_poll<T, E>(
    future: Option<&mut RustFuture<T, E>>,
    waker: Option<RustFutureForeignWakerFunction>,
    waker_environment: *const RustFutureForeignWakerEnvironment,
    polled_result: &mut T::FfiType,
    call_status: &mut RustCallStatus,
) -> bool
where
    T: FfiReturn,
    E: FfiError + FfiConverter<RustType = E, FfiType = RustBuffer>,
{
    // If polling the future panics, an error will be recorded in call_status and the future will
    // be dropped, so there is no potential for observing any inconsistent state in it.
    let mut future = panic::AssertUnwindSafe(future.expect("`future` is a null pointer"));
    let waker = waker.expect("`waker` is a null pointer");

    match call_with_result(call_status, move || {
        future
            .poll(waker, waker_environment)
            .transpose()
            .map_err(E::lower)
    }) {
        RustFuturePoll::Ready(ready) => {
            *polled_result = T::lower(ready);

            READY
        }

        RustFuturePoll::Pending => PENDING,

        RustFuturePoll::Throwing => READY,
    }
}

/// Drop a [`RustFuture`].
///
/// Because this function takes ownership of `future` (by `Box`ing it), the
/// future will be dropped naturally by the compiler at the end of the function
/// scope.
///
/// Please see the module documentation to learn more.
pub fn uniffi_rustfuture_drop<T, E>(
    _future: Option<Box<RustFuture<T, E>>>,
    _call_status: &mut RustCallStatus,
) {
}
