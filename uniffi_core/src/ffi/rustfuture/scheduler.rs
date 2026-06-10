/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem;
use std::sync::Mutex;

use crate::{RustFutureContinuationBoundCallback, RustFuturePoll};

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
///
/// All callbacks are invoked outside the internal lock to prevent ABBA deadlocks with foreign
/// runtime locks. See `wake()` for more details.

#[derive(Debug)]
enum State<Callback> {
    /// No continuations set, neither wake() nor cancel() called.
    Empty,
    /// `wake()` was called when there was no continuation set.  The next time `store` is called,
    /// the continuation should be immediately invoked with `RustFuturePoll::Wake`
    Waked,
    /// The future has been cancelled, any future `store` calls should immediately result in the
    /// continuation being called with `RustFuturePoll::Ready`.
    Cancelled,
    /// Continuation set, the next time `wake()`  is called is called, we should invoke it.
    Set(Callback),
}

pub struct Scheduler<Callback = RustFutureContinuationBoundCallback> {
    state: Mutex<State<Callback>>,
}

/// Callback function that the scheduler stores
pub trait RustFutureCallback {
    fn invoke(self, poll: RustFuturePoll);
}

impl RustFutureCallback for RustFutureContinuationBoundCallback {
    fn invoke(self, poll: RustFuturePoll) {
        (self.callback)(self.data, poll)
    }
}

impl<Callback: RustFutureCallback> Default for Scheduler<Callback> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Callback: RustFutureCallback> Scheduler<Callback> {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State::Empty),
        }
    }

    /// Store new continuation data if we are in the `Empty` state.  If we are in the `Waked` or
    /// `Cancelled` state, call the continuation immediately with the data.
    pub fn store(&self, callback: Callback) {
        let to_invoke = {
            let mut state = self.state.lock().unwrap();

            match *state {
                State::Empty => {
                    *state = State::Set(callback);
                    None
                }
                State::Set(_) => {
                    trace!(
                        "store: observed `Self::Set` state.  Is poll() being called from multiple threads at once?"
                    );
                    let State::Set(old_callback) = mem::replace(&mut *state, State::Set(callback))
                    else {
                        unreachable!();
                    };
                    Some((old_callback, RustFuturePoll::Wake))
                }
                State::Waked => {
                    *state = State::Empty;
                    Some((callback, RustFuturePoll::Wake))
                }
                State::Cancelled => Some((callback, RustFuturePoll::Ready)),
            }
        };

        if let Some((cb, poll)) = to_invoke {
            cb.invoke(poll);
        }
    }

    /// Wake the scheduler.
    ///
    /// If a continuation callback is stored, it will be invoked with `RustFuturePoll::Wake`.
    /// The callback is always invoked after releasing the internal lock to prevenet ABBA
    /// deadlocks: the callback crosses the FFI into a foreign runtime that may need to acquire
    /// a runtime lock (e.g. Ruby GVL or similar), while the foreign calling thread holds that
    /// runtime lock and may call cancel/free (which require this lock).
    pub fn wake(&self) {
        let callback = {
            let mut state = self.state.lock().unwrap();

            match *state {
                // If we had a continuation set, then call it and transition to the `Empty` state.
                State::Set(_) => {
                    let State::Set(callback) = mem::replace(&mut *state, State::Empty) else {
                        unreachable!();
                    };
                    Some(callback)
                }
                // If we were in the `Empty` state, then transition to `Waked`.  The next time `store`
                // is called, we will immediately call the continuation.
                State::Empty => {
                    *state = State::Waked;
                    None
                }
                // This is a no-op if we were in the `Cancelled` or `Waked` state.
                _ => None,
            }
        };

        if let Some(cb) = callback {
            cb.invoke(RustFuturePoll::Wake);
        }
    }

    pub fn cancel(&self) {
        let callback = {
            let mut state = self.state.lock().unwrap();
            match mem::replace(&mut *state, State::Cancelled) {
                State::Set(cb) => Some(cb),
                _ => None,
            }
        };

        if let Some(cb) = callback {
            cb.invoke(RustFuturePoll::Ready);
        }
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(*self.state.lock().unwrap(), State::Cancelled)
    }
}
