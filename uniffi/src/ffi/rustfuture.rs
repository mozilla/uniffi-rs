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
