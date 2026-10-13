/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module defines a Rust Future that wraps an async foreign function call.
//!
//! The general idea is to create a oneshot channel, hand the sender to the foreign side, and
//! await the receiver side on the Rust side.
//!
//! The foreign side should:
//!   * Input a [ForeignFutureCallback] and a `u64` handle in their scaffolding function.
//!     This is the sender, converted to a raw pointer, and an extern "C" function that sends the result.
//!   * Call the [ForeignFutureCallback] when the async function completes with:
//!     * The `u64` handle initially passed in
//!     * The `ForeignFutureResult` struct for the call
//!   * Optionally, set the `ForeignFutureDroppedCallback` value if you want to be notified when the
//!     Future is dropped in Rust.  For languages that support it, this can be hooked up to
//!     cancelling the async task for the method.

use crate::{oneshot, LiftReturn, RustCallStatus};

/// Callback that's passed to a foreign async functions.
///
/// See `LiftReturn` trait for how this is implemented.
pub type ForeignFutureCallback<FfiType> =
    extern "C" fn(oneshot_handle: u64, ForeignFutureResult<FfiType>);

/// C struct that represents the result of a foreign future
#[repr(C)]
pub struct ForeignFutureResult<T> {
    // Note: for void returns, T is `()`, which isn't directly representable with C since it's a ZST.
    // Foreign code should treat that case as if there was no `return_value` field.
    return_value: T,
    call_status: RustCallStatus,
}

/// C callback function that's called when the Rust side of a foreign future is dropped
pub type ForeignFutureDroppedCallback = extern "C" fn(data: u64);

/// C struct that represents a foreign future dropped callback.
#[repr(C)]
pub struct ForeignFutureDroppedCallbackStruct {
    pub callback_data: u64,
    pub callback: ForeignFutureDroppedCallback,
}

impl Default for ForeignFutureDroppedCallbackStruct {
    /// The default value implements a no-op callback.
    /// This will be used for languages that don't set their own callbacks.
    fn default() -> Self {
        extern "C" fn callback(_data: u64) {}
        Self {
            callback_data: 0,
            callback,
        }
    }
}

impl Drop for ForeignFutureDroppedCallbackStruct {
    fn drop(&mut self) {
        (self.callback)(self.callback_data)
    }
}

unsafe impl Send for ForeignFutureDroppedCallbackStruct {}

pub async fn foreign_async_call<F, T, UT>(call_scaffolding_function: F) -> T
where
    F: FnOnce(ForeignFutureCallback<T::ReturnType>, u64, &mut ForeignFutureDroppedCallbackStruct),
    T: LiftReturn<UT>,
{
    // Create a oneshot channel that will receive the result of the callback method
    let (sender, receiver) = oneshot::channel::<ForeignFutureResult<T::ReturnType>>();
    // Create complete callback/data from the oneshot channel
    let complete_callback = foreign_future_complete::<T, UT>;
    let complete_callback_data = sender.into_raw() as u64;
    // Create a `ForeignFutureDroppedCallback`.
    // Since this is owned by the `Future` that we're generating, when the future is dropped it
    // will be dropped too.
    let mut foreign_future_dropped_callback = ForeignFutureDroppedCallbackStruct::default();
    // Call the async method
    call_scaffolding_function(
        complete_callback,
        complete_callback_data,
        &mut foreign_future_dropped_callback,
    );
    // Await the result and use it to return a value
    let result = receiver.await;
    T::lift_foreign_return(result.return_value, result.call_status)
}

pub extern "C" fn foreign_future_complete<T: LiftReturn<UT>, UT>(
    oneshot_handle: u64,
    result: ForeignFutureResult<T::ReturnType>,
) {
    let channel = unsafe { oneshot::Sender::from_raw(oneshot_handle as *mut ()) };
    channel.send(result);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Lower, RustBuffer};
    use once_cell::sync::OnceCell;
    use std::{
        future::Future,
        pin::Pin,
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
        task::{Context, Poll, Wake},
    };

    struct MockForeignFuture {
        freed: Arc<AtomicU32>,
        callback_info: Arc<OnceCell<(ForeignFutureCallback<RustBuffer>, u64)>>,
        rust_future: Option<Pin<Box<dyn Future<Output = String>>>>,
    }

    impl MockForeignFuture {
        fn new() -> Self {
            let callback_info = Arc::new(OnceCell::new());
            let future_dropped_call_count = Arc::new(AtomicU32::new(0));

            let rust_future: Pin<Box<dyn Future<Output = String>>> = {
                let callback_info = callback_info.clone();
                let future_dropped_call_count = future_dropped_call_count.clone();
                Box::pin(foreign_async_call::<_, String, crate::UniFfiTag>(
                    move |callback, data, out_dropped_callback| {
                        callback_info.set((callback, data)).unwrap();
                        *out_dropped_callback = ForeignFutureDroppedCallbackStruct {
                            callback_data: Arc::into_raw(future_dropped_call_count) as *mut ()
                                as u64,
                            callback: Self::future_drapped_callback,
                        };
                    },
                ))
            };
            let rust_future = Some(rust_future);
            let mut mock_foreign_future = Self {
                freed: future_dropped_call_count,
                callback_info,
                rust_future,
            };
            // Poll the future once, to start it up.   This ensures that `callback_info` is set.
            let _ = mock_foreign_future.poll();
            mock_foreign_future
        }

        fn poll(&mut self) -> Poll<String> {
            let waker = Arc::new(NoopWaker).into();
            let mut context = Context::from_waker(&waker);
            self.rust_future
                .as_mut()
                .unwrap()
                .as_mut()
                .poll(&mut context)
        }

        fn complete_success(&self, value: String) {
            let (callback, data) = self.callback_info.get().unwrap();
            callback(
                *data,
                ForeignFutureResult {
                    return_value: <String as Lower<crate::UniFfiTag>>::lower(value),
                    call_status: RustCallStatus::default(),
                },
            );
        }

        fn complete_error(&self, error_message: String) {
            let (callback, data) = self.callback_info.get().unwrap();
            callback(
                *data,
                ForeignFutureResult {
                    return_value: RustBuffer::default(),
                    call_status: RustCallStatus::error(error_message),
                },
            );
        }

        fn drop_future(&mut self) {
            self.rust_future = None
        }

        fn free_count(&self) -> u32 {
            self.freed.load(Ordering::Relaxed)
        }

        extern "C" fn future_drapped_callback(handle: u64) {
            let flag = unsafe { Arc::from_raw(handle as *mut AtomicU32) };
            flag.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct NoopWaker;

    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn test_foreign_future() {
        let mut mock_foreign_future = MockForeignFuture::new();
        assert_eq!(mock_foreign_future.poll(), Poll::Pending);
        mock_foreign_future.complete_success("It worked!".to_owned());
        assert_eq!(
            mock_foreign_future.poll(),
            Poll::Ready("It worked!".to_owned())
        );
        // Since the future is complete, it should free the foreign future
        assert_eq!(mock_foreign_future.free_count(), 1);
    }

    #[test]
    #[should_panic]
    fn test_foreign_future_error() {
        let mut mock_foreign_future = MockForeignFuture::new();
        assert_eq!(mock_foreign_future.poll(), Poll::Pending);
        mock_foreign_future.complete_error("It Failed!".to_owned());
        let _ = mock_foreign_future.poll();
    }

    #[test]
    fn test_drop_after_complete() {
        let mut mock_foreign_future = MockForeignFuture::new();
        mock_foreign_future.complete_success("It worked!".to_owned());
        assert_eq!(mock_foreign_future.free_count(), 0);
        assert_eq!(
            mock_foreign_future.poll(),
            Poll::Ready("It worked!".to_owned())
        );
        // Dropping the future after it's complete should not panic, and not cause a double-free
        mock_foreign_future.drop_future();
        assert_eq!(mock_foreign_future.free_count(), 1);
    }

    #[test]
    fn test_drop_before_complete() {
        let mut mock_foreign_future = MockForeignFuture::new();
        mock_foreign_future.complete_success("It worked!".to_owned());
        // Dropping the future before it's complete should cancel the future
        assert_eq!(mock_foreign_future.free_count(), 0);
        mock_foreign_future.drop_future();
        assert_eq!(mock_foreign_future.free_count(), 1);
    }
}
