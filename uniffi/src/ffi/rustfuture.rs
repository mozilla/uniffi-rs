use crate::{call_with_output, FfiConverter, RustCallStatus};

use super::FfiDefault;
use std::{
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
pub struct RustFuture<T>(
    Pin<Box<dyn Future<Output = <T as FfiConverter>::RustType> + Send + 'static>>,
)
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
        F: Future<Output = <T as FfiConverter>::RustType> + Send + 'static,
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
    pub fn poll(
        &mut self,
        foreign_waker_pointer: *const RustFutureForeignWaker,
    ) -> Poll<<T as FfiConverter>::FfiType> {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(Arc::new(foreign_waker_pointer)) as *const (),
                &RustTaskWakerBuilder::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        Pin::new(&mut self.0)
            .poll(&mut context)
            .map(|result| <T as FfiConverter>::lower(result))
    }
}

impl<T> FfiDefault for RustFuture<T>
where
    T: FfiConverter,
{
    fn ffi_default() -> Self {
        Self::new(async { unreachable!("A default future must not be polled") })
    }
}

impl<T> FfiDefault for Box<RustFuture<T>>
where
    T: FfiConverter,
{
    fn ffi_default() -> Self {
        Box::new(RustFuture::ffi_default())
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
pub type RustFutureForeignWaker = extern "C" fn();

/// Zero-sized type to create the VTable for the `RawWaker`.
struct RustTaskWakerBuilder;

impl RustTaskWakerBuilder {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(foreign_waker: *const ()) -> RawWaker {
        Arc::increment_strong_count(foreign_waker);

        RawWaker::new(foreign_waker, &Self::VTABLE)
    }

    unsafe fn wake(foreign_waker: *const ()) {
        let waker: *const RustFutureForeignWaker = mem::transmute(foreign_waker);
        let waker = Arc::from_raw(waker);
        (waker)();
    }

    unsafe fn wake_by_ref(foreign_waker: *const ()) {
        let waker: *const RustFutureForeignWaker = mem::transmute(foreign_waker);
        let waker = ManuallyDrop::new(Arc::from_raw(waker));
        (waker)();
    }

    unsafe fn drop_waker(foreign_waker: *const ()) {
        let waker: *const RustFutureForeignWaker = mem::transmute(foreign_waker);
        drop(Arc::from_raw(waker));
    }
}

#[doc(hidden)]
pub fn uniffi_rustfuture_poll<T>(
    future: Option<&mut RustFuture<T>>,
    waker: Option<NonNull<RustFutureForeignWaker>>,
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
        future_mutex.lock().unwrap().poll(waker.as_ptr())
    }) {
        Poll::Ready(result) => {
            *polled_result = result;

            true
        }

        Poll::Pending => false,
    }
}

#[doc(hidden)]
pub fn uniffi_rustfuture_drop<T>(
    _future: Option<Box<RustFuture<T>>>,
    _call_status: &mut RustCallStatus,
) where
    T: FfiConverter,
{
}
