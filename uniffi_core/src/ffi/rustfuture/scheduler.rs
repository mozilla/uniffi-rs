/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{mem, sync::Mutex};

use super::{RustFutureContinuationCallback, RustFuturePoll};

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
pub(super) struct Scheduler {
    state: Mutex<SchedulerState>,
}

#[derive(Debug)]
pub(super) enum SchedulerState {
    /// No continuations set, neither wake() nor cancel() called.
    Empty,
    /// `wake()` was called when there was no continuation set.  The next time `store` is called,
    /// the continuation should be immediately invoked with `RustFuturePoll::MaybeReady`
    Waked,
    /// The future has been cancelled, any future `store` calls should immediately result in the
    /// continuation being called with `RustFuturePoll::Ready`.
    Cancelled,
    /// Continuation set, the next time `wake()`  is called is called, we should invoke it.
    Set(RustFutureContinuationCallback, *const ()),
}

/// Encapsulates a call to a RustFutureContinuationCallback
struct CallbackCall {
    callback: RustFutureContinuationCallback,
    data: *const (),
    poll_data: RustFuturePoll,
}

impl CallbackCall {
    fn new(
        callback: RustFutureContinuationCallback,
        data: *const (),
        poll_data: RustFuturePoll,
    ) -> Self {
        Self {
            callback,
            data,
            poll_data,
        }
    }

    fn invoke(self) {
        (self.callback)(self.data, self.poll_data)
    }
}

/// The SchedulerState impl contains all the ways to mutate the inner state field.
///
/// All methods return an `Option<CallbackCall>` rather than invoking the callback directly.
/// This is important, since the Mutex is locked while inside these methods.  If we called the
/// callback directly, the foreign code could poll the future again, which would try to lock the
/// mutex again and lead to a deadlock.
impl SchedulerState {
    fn store(
        &mut self,
        callback: RustFutureContinuationCallback,
        data: *const (),
    ) -> Option<CallbackCall> {
        match self {
            Self::Empty => {
                *self = Self::Set(callback, data);
                None
            }
            Self::Set(old_callback, old_data) => {
                log::error!(
                    "store: observed `SchedulerState::Set` state.  Is poll() being called from multiple threads at once?"
                );
                let call = CallbackCall::new(*old_callback, *old_data, RustFuturePoll::MaybeReady);
                *self = Self::Set(callback, data);
                Some(call)
            }
            Self::Waked => {
                *self = Self::Empty;
                Some(CallbackCall::new(
                    callback,
                    data,
                    RustFuturePoll::MaybeReady,
                ))
            }
            Self::Cancelled => Some(CallbackCall::new(callback, data, RustFuturePoll::Ready)),
        }
    }

    fn wake(&mut self) -> Option<CallbackCall> {
        match self {
            // If we had a continuation set, then call it and transition to the `Empty` state.
            SchedulerState::Set(callback, old_data) => {
                let old_data = *old_data;
                let callback = *callback;
                *self = SchedulerState::Empty;
                Some(CallbackCall::new(
                    callback,
                    old_data,
                    RustFuturePoll::MaybeReady,
                ))
            }
            // If we were in the `Empty` state, then transition to `Waked`.  The next time `store`
            // is called, we will immediately call the continuation.
            SchedulerState::Empty => {
                *self = SchedulerState::Waked;
                None
            }
            // This is a no-op if we were in the `Cancelled` or `Waked` state.
            _ => None,
        }
    }

    fn cancel(&mut self) -> Option<CallbackCall> {
        if let SchedulerState::Set(callback, old_data) =
            mem::replace(self, SchedulerState::Cancelled)
        {
            Some(CallbackCall::new(callback, old_data, RustFuturePoll::Ready))
        } else {
            None
        }
    }
}

impl Scheduler {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(SchedulerState::Empty),
        }
    }

    /// Call a method on the inner state field
    ///
    /// If it returns a callback to invoke, then make the call after releasing the mutex.
    fn call_state_method(&self, f: impl Fn(&mut SchedulerState) -> Option<CallbackCall>) {
        let mut state = self.state.lock().unwrap();
        let callback_call = f(&mut state);
        drop(state);
        if let Some(callback_call) = callback_call {
            callback_call.invoke()
        }
    }

    /// Store new continuation data if we are in the `Empty` state.  If we are in the `Waked` or
    /// `Cancelled` state, call the continuation immediately with the data.
    pub(super) fn store(&self, callback: RustFutureContinuationCallback, data: *const ()) {
        self.call_state_method(|state| state.store(callback, data))
    }

    pub(super) fn wake(&self) {
        self.call_state_method(SchedulerState::wake)
    }

    pub(super) fn cancel(&self) {
        self.call_state_method(SchedulerState::cancel)
    }

    pub(super) fn is_cancelled(&self) -> bool {
        matches!(*self.state.lock().unwrap(), SchedulerState::Cancelled)
    }
}

// The `*const ()` data pointer references an object on the foreign side.
// This object must be `Sync` in Rust terminology -- it must be safe for us to pass the pointer to the continuation callback from any thread.
// If the foreign side upholds their side of the contract, then `Scheduler` is Send + Sync.

unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}
