/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    future::Future,
    panic,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{rust_call_with_out_status, FfiDefault, LowerReturn, RustCallStatus};

/// Wraps the actual future we're polling
pub struct WrappedFuture<F, T, UT>
where
    // See the [RustFuture] struct for an explanation of these trait bounds
    F: Future<Output = T> + Send + 'static,
    T: LowerReturn<UT> + Send + 'static,
    UT: Send + 'static,
{
    // Note: this could be a single enum, but that would make it easy to mess up the future pinning
    // guarantee.   For example you might want to call `std::mem::take()` to try to get the result,
    // but if the future happened to be stored that would move and break all internal references.
    future: Option<F>,
    result: Option<Result<T::ReturnType, RustCallStatus>>,
}

impl<F, T, UT> WrappedFuture<F, T, UT>
where
    // See the [RustFuture] struct for an explanation of these trait bounds
    F: Future<Output = T> + Send + 'static,
    T: LowerReturn<UT> + Send + 'static,
    UT: Send + 'static,
{
    pub fn new(future: F) -> Self {
        Self {
            future: Some(future),
            result: None,
        }
    }

    // Poll the future and check if it's ready or not
    pub fn poll(&mut self, context: &mut Context<'_>) -> bool {
        if self.result.is_some() {
            true
        } else if let Some(future) = &mut self.future {
            // SAFETY: We can call Pin::new_unchecked because:
            //    - This is the only time we get a &mut to `self.future`
            //    - We never poll the future after it's moved (for example by using take())
            //    - We never move RustFuture, which contains us.
            //    - RustFuture is private to this module so no other code can move it.
            let pinned = unsafe { Pin::new_unchecked(future) };
            // Run the poll and lift the result if it's ready
            let mut out_status = RustCallStatus::default();
            let result: Option<Poll<T::ReturnType>> = rust_call_with_out_status(
                &mut out_status,
                // This closure uses a `&mut F` value, which means it's not UnwindSafe by
                // default.  If the future panics, it may be in an invalid state.
                //
                // However, we can safely use `AssertUnwindSafe` since a panic will lead the `None`
                // case below and we will never poll the future again.
                panic::AssertUnwindSafe(|| match pinned.poll(context) {
                    Poll::Pending => Ok(Poll::Pending),
                    Poll::Ready(v) => T::lower_return(v).map(Poll::Ready),
                }),
            );
            match result {
                Some(Poll::Pending) => false,
                Some(Poll::Ready(v)) => {
                    self.future = None;
                    self.result = Some(Ok(v));
                    true
                }
                None => {
                    self.future = None;
                    self.result = Some(Err(out_status));
                    true
                }
            }
        } else {
            log::error!("poll with neither future nor result set");
            true
        }
    }

    pub fn complete(&mut self, out_status: &mut RustCallStatus) -> T::ReturnType {
        let mut return_value = T::ReturnType::ffi_default();
        match self.result.take() {
            Some(Ok(v)) => return_value = v,
            Some(Err(call_status)) => *out_status = call_status,
            None => *out_status = RustCallStatus::cancelled(),
        }
        self.free();
        return_value
    }

    pub fn free(&mut self) {
        self.future = None;
        self.result = None;
    }
}

// If F and T are Send, then WrappedFuture is too
//
// Rust will not mark it Send by default when T::ReturnType is a raw pointer.  This is promising
// that we will treat the raw pointer properly, for example by not returning it twice.
unsafe impl<F, T, UT> Send for WrappedFuture<F, T, UT>
where
    // See the [RustFuture] struct for an explanation of these trait bounds
    F: Future<Output = T> + Send + 'static,
    T: LowerReturn<UT> + Send + 'static,
    UT: Send + 'static,
{
}
