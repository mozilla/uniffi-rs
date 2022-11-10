use super::FfiDefault;
use std::{
    future::{self, Future},
    mem::{self, ManuallyDrop},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

#[repr(transparent)]
pub struct RustFuture<T>(Pin<Box<dyn Future<Output = T> + Send + 'static>>)
where
    T: Send + 'static;

impl<T> RustFuture<T>
where
    T: Send + 'static,
{
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self(Box::pin(future))
    }

    pub fn poll(&mut self, waker_pointer: *const RustFutureForeignWaker) -> bool {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(Arc::new(waker_pointer)) as *const (),
                &RustTaskWakerBuilder::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        match Pin::new(&mut self.0).poll(&mut context) {
            Poll::Ready(_result) => true,
            Poll::Pending => false,
        }
    }
}

impl<T> FfiDefault for RustFuture<T>
where
    T: Send + 'static + FfiDefault,
{
    fn ffi_default() -> Self {
        Self::new(future::ready(T::ffi_default()))
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
