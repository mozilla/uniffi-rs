use super::FfiDefault;
use std::{
    future::{self, Future},
    mem::{self, ManuallyDrop},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

#[repr(transparent)]
pub struct FfiFuture<T>(Pin<Box<dyn Future<Output = T> + Send + 'static>>)
where
    T: Send + 'static;

impl<T> FfiFuture<T>
where
    T: Send + 'static,
{
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        Self(Box::pin(future))
    }

    pub fn poll(&mut self, waker_pointer: usize) -> Option<T> {
        let waker: Waker = { unimplemented!() };
        let context: Context = Context::from_waker(&waker);

        match Pin::new(&mut self.0).poll(&mut context) {
            Poll::Ready(result) => Some(result),
            Poll::Pending => None,
        }
    }
}

impl<T> FfiDefault for FfiFuture<T>
where
    T: Send + 'static + FfiDefault,
{
    fn ffi_default() -> Self {
        Self::new(future::ready(T::ffi_default()))
    }
}

struct FfiTaskWakerBuilder<F>(F)
where
    F: Fn() + Send + Sync + 'static;

impl<F> FfiTaskWakerBuilder<F>
where
    F: Fn() + Send + Sync + 'static,
{
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        mem::forget(arc.clone());

        RawWaker::new(ptr, &Self::VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const F);
        (arc)();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        (arc)();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const F));
    }
}
