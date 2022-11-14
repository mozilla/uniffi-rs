use crate::{call_with_output, RustBuffer, RustCallStatus};

use super::FfiDefault;
use std::{
    future::{self, Future},
    mem::{self, ManuallyDrop},
    pin::Pin,
    ptr::NonNull,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

/// `RustFuture` must have the size of a pointer.
#[repr(C)]
pub struct RustFuture(Box<Mutex<Pin<Box<dyn Future<Output = RustBuffer> + Send + 'static>>>>);

impl RustFuture {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = RustBuffer> + Send + 'static,
    {
        Self(Box::new(Mutex::new(Box::pin(future))))
    }

    pub fn poll(&mut self, waker_pointer: *const RustFutureForeignWaker) -> bool {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(Arc::new(waker_pointer)) as *const (),
                &RustTaskWakerBuilder::VTABLE,
            ))
        };
        let mut context = Context::from_waker(&waker);

        let future = self.0.get_mut().expect("arf");

        match Pin::new(future).poll(&mut context) {
            Poll::Ready(_result) => true,
            Poll::Pending => false,
        }
    }
}

impl FfiDefault for RustFuture {
    fn ffi_default() -> Self {
        Self::new(async { RustBuffer::default() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_future_size() {
        assert_eq!(
            mem::size_of_val(&RustFuture::new(async { unreachable!() })),
            mem::size_of::<*const u8>(),
        );
    }
}

// #[repr(transparent)]
// pub struct RustFuture<T>(Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>)
// where
//     T: Send + 'static;

// impl<T> RustFuture<T>
// where
//     T: Send + 'static,
// {
//     pub fn new<F>(future: F) -> Self
//     where
//         F: Future<Output = T> + Send + 'static,
//     {
//         Self(Mutex::new(Box::pin(future)))
//     }

//     pub fn poll(&mut self, waker_pointer: *const RustFutureForeignWaker) -> bool {
//         let waker = unsafe {
//             Waker::from_raw(RawWaker::new(
//                 Arc::into_raw(Arc::new(waker_pointer)) as *const (),
//                 &RustTaskWakerBuilder::VTABLE,
//             ))
//         };
//         let mut context = Context::from_waker(&waker);
//         let future = self.0.get_mut().unwrap();

//         match Pin::new(future).poll(&mut context) {
//             Poll::Ready(_result) => true,
//             Poll::Pending => false,
//         }
//     }
// }

// impl<T> FfiDefault for RustFuture<T>
// where
//     T: Send + 'static + FfiDefault,
// {
//     fn ffi_default() -> Self {
//         Self::new(future::ready(T::ffi_default()))
//     }
// }

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

#[no_mangle]
pub unsafe extern "C" fn uniffi_rustfuture_poll(
    future: Option<NonNull<RustFuture>>,
    waker: Option<NonNull<RustFutureForeignWaker>>,
    call_status: &mut RustCallStatus,
) -> bool {
    let future: NonNull<RustFuture> = future.expect("`future` is a null pointer");
    let waker: NonNull<RustFutureForeignWaker> = waker.expect("`waker` is a null pointer");

    let future: *mut RustFuture = future.as_ptr();

    call_with_output(call_status, || {
        future.as_mut().unwrap().poll(waker.as_ptr())
    })
}
