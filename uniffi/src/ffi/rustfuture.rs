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

/// `RustFuture` must have the size of a pointer.
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
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = <T as FfiConverter>::RustType> + Send + 'static,
    {
        Self(Box::pin(future))
    }

    pub fn poll(
        &mut self,
        waker_pointer: *const RustFutureForeignWaker,
    ) -> Option<<T as FfiConverter>::FfiType> {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(Arc::new(waker_pointer)) as *const (),
                &RustTaskWakerBuilder::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        match Pin::new(&mut self.0).poll(&mut context) {
            Poll::Ready(result) => Some(<T as FfiConverter>::lower(result)),
            Poll::Pending => None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_future_size() {
        assert_eq!(
            mem::size_of_val(&RustFuture::<bool>::new(async { unreachable!() })),
            mem::size_of::<*const u8>(),
        );
    }
}

pub type RustFutureForeignWaker = extern "C" fn();

struct RustTaskWakerBuilder;

impl RustTaskWakerBuilder {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(foreign_waker: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::from_raw(foreign_waker));
        mem::forget(arc.clone());

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
    call_status: &mut RustCallStatus,
) -> Option<<T as FfiConverter>::FfiType>
where
    T: FfiConverter,
{
    let future = future.expect("`future` is a null pointer");
    let waker = waker.expect("`waker` is a null pointer");

    let future_mutex = Mutex::new(future);

    call_with_output(call_status, || {
        future_mutex.lock().unwrap().poll(waker.as_ptr())
    })
}

#[doc(hidden)]
pub fn uniffi_rustfuture_drop<T>(
    _future: Option<Box<RustFuture<T>>>,
    _call_status: &mut RustCallStatus,
) where
    T: FfiConverter,
{
}
