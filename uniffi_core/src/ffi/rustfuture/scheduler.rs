/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, future::poll_fn, mem, num::NonZeroU64, task::Poll, thread_local};

use super::{RustFutureContinuationCallback, RustFuturePoll};

/// Context of the current `RustFuture::poll` call
struct RustFutureContext {
    /// Blocking task queue that the future is being polled on
    current_blocking_task_queue_handle: Option<NonZeroU64>,
    /// Blocking task queue that we've been asked to schedule the next poll on
    scheduled_blocking_task_queue_handle: Option<NonZeroU64>,
}

thread_local! {
    static CONTEXT: RefCell<RustFutureContext> = RefCell::new(RustFutureContext {
        current_blocking_task_queue_handle: None,
        scheduled_blocking_task_queue_handle: None,
    });
}

fn with_context<F: FnOnce(&mut RustFutureContext) -> R, R>(operation: F) -> R {
    CONTEXT.with(|context| operation(&mut context.borrow_mut()))
}

pub fn on_poll_start(current_blocking_task_queue_handle: Option<NonZeroU64>) {
    with_context(|context| {
        *context = RustFutureContext {
            current_blocking_task_queue_handle,
            scheduled_blocking_task_queue_handle: None,
        }
    });
}

pub fn on_poll_end() {
    with_context(|context| {
        *context = RustFutureContext {
            current_blocking_task_queue_handle: None,
            scheduled_blocking_task_queue_handle: None,
        }
    });
}

/// Schedule work in a blocking task queue
///
/// The returned future will attempt to arrange for [RustFuture::poll] to be called in the
/// blocking task queue.  Once [RustFuture::poll] is running in the blocking task queue, then the future
/// will be ready.
///
/// There's one tricky issue here: how can we ensure that when the top-level task is run in the
/// blocking task queue, this future will be polled?  What happens this future is a child of `join!`,
/// `FuturesUnordered` or some other Future that handles its own polling?
///
/// We start with an assumption: if we notify the waker then this future will be polled when the
/// top-level task is polled next.  If a future does not honor this then we consider it a broken
/// future.  This seems fair, since that future would almost certainly break a lot of other future
/// code.
///
/// Based on that, we can have a simple system.  When we're polled:
///   * If we're running in the blocking task queue, then we return `Poll::Ready`.
///   * If not, we return `Poll::Pending` and notify the waker so that the future polls again on
///     the next top-level poll.
///
/// Note that this can be inefficient if the code awaits multiple blocking task queues at once.  We
/// can only run the next poll on one of them, but all futures will be woken up.  This seems okay
/// for our intended use cases, it would be pretty odd for a library to use multiple blocking task
/// queues.  The alternative would be to store the set of all pending blocking task queues, which
/// seems like complete overkill for our purposes.
pub(super) async fn schedule_in_blocking_task_queue(handle: NonZeroU64) {
    poll_fn(|future_context| {
        with_context(|poll_context| {
            if poll_context.current_blocking_task_queue_handle == Some(handle) {
                Poll::Ready(())
            } else {
                poll_context
                    .scheduled_blocking_task_queue_handle
                    .get_or_insert(handle);
                future_context.waker().wake_by_ref();
                Poll::Pending
            }
        })
    })
    .await
}

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
pub(super) enum Scheduler {
    /// No continuations set, neither wake() nor cancel() called.
    Empty,
    /// `wake()` was called when there was no continuation set.  The next time `store` is called,
    /// the continuation should be immediately invoked with `RustFuturePoll::MaybeReady`
    Waked,
    /// The future has been cancelled, any future `store` calls should immediately result in the
    /// continuation being called with `RustFuturePoll::Ready`.
    Cancelled,
    /// Continuation set, the next time `wake()`  is called is called, we should invoke it.
    Set(RustFutureContinuationCallback, u64),
}

impl Scheduler {
    pub(super) fn new() -> Self {
        Self::Empty
    }

    /// Store new continuation data if we are in the `Empty` state.  If we are in the `Waked` or
    /// `Cancelled` state, call the continuation immediately with the data.
    pub(super) fn store(&mut self, callback: RustFutureContinuationCallback, data: u64) {
        if let Some(blocking_task_queue_handle) =
            with_context(|context| context.scheduled_blocking_task_queue_handle)
        {
            // We were asked to schedule the future in a blocking task queue, call the callback
            // rather than storing it
            callback(
                data,
                RustFuturePoll::MaybeReady,
                blocking_task_queue_handle.into(),
            );
            return;
        }

        match self {
            Self::Empty => *self = Self::Set(callback, data),
            Self::Set(old_callback, old_data) => {
                log::error!(
                    "store: observed `Self::Set` state.  Is poll() being called from multiple threads at once?"
                );
                old_callback(*old_data, RustFuturePoll::Ready, 0);
                *self = Self::Set(callback, data);
            }
            Self::Waked => {
                *self = Self::Empty;
                callback(data, RustFuturePoll::MaybeReady, 0);
            }
            Self::Cancelled => {
                callback(data, RustFuturePoll::Ready, 0);
            }
        }
    }

    pub(super) fn wake(&mut self) {
        match self {
            // If we had a continuation set, then call it and transition to the `Empty` state.
            Self::Set(callback, old_data) => {
                let old_data = *old_data;
                let callback = *callback;
                *self = Self::Empty;
                callback(old_data, RustFuturePoll::MaybeReady, 0);
            }
            // If we were in the `Empty` state, then transition to `Waked`.  The next time `store`
            // is called, we will immediately call the continuation.
            Self::Empty => *self = Self::Waked,
            // This is a no-op if we were in the `Cancelled` or `Waked` state.
            _ => (),
        }
    }

    pub(super) fn cancel(&mut self) {
        if let Self::Set(callback, old_data) = mem::replace(self, Self::Cancelled) {
            callback(old_data, RustFuturePoll::Ready, 0);
        }
    }

    pub(super) fn clear_wake_flag(&mut self) {
        if let Self::Waked = self {
            *self = Self::Empty
        }
    }

    pub(super) fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

// The `*const ()` data pointer references an object on the foreign side.
// This object must be `Sync` in Rust terminology -- it must be safe for us to pass the pointer to the continuation callback from any thread.
// If the foreign side upholds their side of the contract, then `Scheduler` is Send + Sync.

unsafe impl Send for Scheduler {}
unsafe impl Sync for Scheduler {}
