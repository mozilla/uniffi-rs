/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem;

use crate::{RustFutureContinuationCallback, RustFuturePoll};

/// Schedules a [crate::RustFuture] by managing the continuation data
///
/// This struct manages the continuation callback and data that comes from the foreign side.  It
/// is responsible for calling the continuation callback when the future is ready to be woken up.
///
/// The basic guarantees are:
///
/// * Each callback will be invoked exactly once, with its associated data.
/// * If `wake()` is called, the callback will be invoked to wake up the future -- either
///   immediately or the next time we get a callback.
/// * If `cancel()` is called, the same will happen and the schedule will stay in the cancelled
///   state, invoking any future callbacks as soon as they're stored.

#[derive(Debug)]
pub(crate) enum Scheduler<Callback = RustFutureContinuationCallback> {
    /// No continuations set, neither wake() nor cancel() called.
    Empty,
    /// `wake()` was called when there was no continuation set.  The next time `store` is called,
    /// the continuation should be immediately invoked with `RustFuturePoll::Wake`
    Waked,
    /// The future has been cancelled, any future `store` calls should immediately result in the
    /// continuation being called with `RustFuturePoll::Ready`.
    Cancelled,
    /// Continuation set, the next time `wake()`  is called is called, we should invoke it.
    Set(Callback, u64),
}

/// Callback function that the scheduler stores
///
/// This will vary based on if we're using the legacy FFI or pointer FFI
pub(crate) trait RustFutureCallback {
    fn invoke(self, data: u64, poll: RustFuturePoll);
}

impl RustFutureCallback for RustFutureContinuationCallback {
    fn invoke(self, data: u64, poll: RustFuturePoll) {
        self(data, poll)
    }
}

impl<Callback: RustFutureCallback> Scheduler<Callback> {
    pub(crate) fn new() -> Self {
        Self::Empty
    }

    /// Store new continuation data if we are in the `Empty` state.  If we are in the `Waked` or
    /// `Cancelled` state, call the continuation immediately with the data.
    pub(crate) fn store(&mut self, callback: Callback, data: u64) {
        match self {
            Self::Empty => *self = Self::Set(callback, data),
            Self::Set(_, _) => {
                trace!(
                    "store: observed `Self::Set` state.  Is poll() being called from multiple threads at once?"
                );
                let Self::Set(old_callback, old_data) =
                    mem::replace(self, Self::Set(callback, data))
                else {
                    unreachable!();
                };
                old_callback.invoke(old_data, RustFuturePoll::Ready);
            }
            Self::Waked => {
                *self = Self::Empty;
                callback.invoke(data, RustFuturePoll::Wake);
            }
            Self::Cancelled => {
                callback.invoke(data, RustFuturePoll::Ready);
            }
        }
    }

    pub(crate) fn wake(&mut self) {
        match self {
            // If we had a continuation set, then call it and transition to the `Empty` state.
            Self::Set(_, _) => {
                let Self::Set(callback, old_data) = mem::replace(self, Self::Empty) else {
                    unreachable!();
                };
                callback.invoke(old_data, RustFuturePoll::Wake);
            }
            // If we were in the `Empty` state, then transition to `Waked`.  The next time `store`
            // is called, we will immediately call the continuation.
            Self::Empty => *self = Self::Waked,
            // This is a no-op if we were in the `Cancelled` or `Waked` state.
            _ => (),
        }
    }

    pub(crate) fn cancel(&mut self) {
        if let Self::Set(callback, old_data) = mem::replace(self, Self::Cancelled) {
            callback.invoke(old_data, RustFuturePoll::Ready);
        }
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

// The `*const ()` data pointer references an object on the foreign side.
// This object must be `Sync` in Rust terminology -- it must be safe for us to pass the pointer to the continuation callback from any thread.
// If the foreign side upholds their side of the contract, then `Scheduler` is Send + Sync.

unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}
