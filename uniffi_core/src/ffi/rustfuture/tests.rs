use once_cell::sync::OnceCell;
use std::{
    cell::Cell,
    future::Future,
    mem::ManuallyDrop,
    panic,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use super::*;
use crate::{test_util::TestError, Lift, RustBuffer, RustCallStatusCode};

// Sender/Receiver pair that we use for testing
struct Channel {
    result: Option<Result<Result<String, TestError>, LiftArgsError>>,
    waker: Option<Waker>,
}

struct Sender(Arc<Mutex<Channel>>);

impl Sender {
    fn wake(&self) {
        let inner = self.0.lock().unwrap();
        if let Some(waker) = &inner.waker {
            waker.wake_by_ref();
        }
    }

    fn send(&self, value: Result<String, TestError>) {
        let mut inner = self.0.lock().unwrap();
        if inner.result.replace(Ok(value)).is_some() {
            panic!("value already sent");
        }
        if let Some(waker) = &inner.waker {
            waker.wake_by_ref();
        }
    }

    fn send_lift_args_error(&self, arg_name: &'static str, error: anyhow::Error) {
        let mut inner = self.0.lock().unwrap();
        if inner
            .result
            .replace(Err(LiftArgsError { arg_name, error }))
            .is_some()
        {
            panic!("value already sent");
        }
        if let Some(waker) = &inner.waker {
            waker.wake_by_ref();
        }
    }
}

struct Receiver(Arc<Mutex<Channel>>);

impl Future for Receiver {
    type Output = Result<Result<String, TestError>, LiftArgsError>;

    fn poll(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<Result<String, TestError>, LiftArgsError>> {
        let mut inner = self.0.lock().unwrap();
        match inner.result.take() {
            Some(v) => Poll::Ready(v),
            None => {
                inner.waker = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

// Create a sender and rust future that we can use for testing
fn channel() -> (Sender, Arc<RustFuture<RustBuffer>>) {
    let channel = Arc::new(Mutex::new(Channel {
        result: None,
        waker: None,
    }));
    let rust_future = RustFuture::new(Box::pin(Receiver(channel.clone())), crate::UniFfiTag);
    (Sender(channel), Arc::new(rust_future))
}

/// Poll a Rust future and get an OnceCell that's set when the continuation is called
fn poll(rust_future: &Arc<RustFuture<RustBuffer>>) -> Arc<OnceCell<RustFuturePoll>> {
    let cell = Arc::new(OnceCell::new());
    let handle = Arc::into_raw(cell.clone()) as u64;
    rust_future
        .clone()
        .poll(RustFutureContinuationBoundCallback {
            callback: poll_continuation,
            data: handle,
        });
    cell
}

extern "C" fn poll_continuation(data: u64, code: RustFuturePoll) {
    let cell = unsafe { Arc::from_raw(data as *const OnceCell<RustFuturePoll>) };
    cell.set(code).expect("Error setting OnceCell");
}

fn complete(rust_future: Arc<RustFuture<RustBuffer>>) -> (RustBuffer, RustCallStatus) {
    let mut out_status_code = RustCallStatus::default();
    let return_value = rust_future.complete(&mut out_status_code);
    (return_value, out_status_code)
}

#[test]
fn test_success() {
    let (sender, rust_future) = channel();

    // Test polling the rust future before it's ready
    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), None);
    sender.wake();
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Wake));

    // Test polling the rust future when it's ready
    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), None);
    sender.send(Ok("All done".into()));
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Wake));

    // Future polls should immediately return ready
    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));

    // Complete the future
    let (return_buf, call_status) = complete(rust_future);
    assert_eq!(call_status.code, RustCallStatusCode::Success);
    assert_eq!(
        <String as Lift<crate::UniFfiTag>>::try_lift(return_buf).unwrap(),
        "All done"
    );
}

#[test]
fn test_error() {
    let (sender, rust_future) = channel();

    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), None);
    sender.send(Err("Something went wrong".into()));
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Wake));

    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));

    let (_, call_status) = complete(rust_future);
    assert_eq!(call_status.code, RustCallStatusCode::Error);
    assert_eq!(
        <TestError as Lift<crate::UniFfiTag>>::try_lift_from_rust_buffer(ManuallyDrop::into_inner(
            call_status.error_buf
        ))
        .unwrap(),
        TestError::from("Something went wrong"),
    )
}

#[test]
fn test_lift_args_error() {
    let (sender, rust_future) = channel();

    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), None);
    sender.send_lift_args_error("arg0", anyhow::anyhow!("Invalid handle"));
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Wake));

    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));

    let (_, call_status) = complete(rust_future);
    assert_eq!(call_status.code, RustCallStatusCode::UnexpectedError);
    assert_eq!(
        <String as Lift<crate::UniFfiTag>>::try_lift(ManuallyDrop::into_inner(
            call_status.error_buf
        ))
        .unwrap(),
        "Failed to convert arg 'arg0':\nInvalid handle",
    )
}

// Once `complete` is called, the inner future should be released, even if wakers still hold a
// reference to the RustFuture
#[test]
fn test_cancel() {
    let (_sender, rust_future) = channel();

    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), None);
    rust_future.cancel();
    // Cancellation should immediately invoke the callback with RustFuturePoll::Ready
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));

    // Future polls should immediately invoke the callback with RustFuturePoll::Ready
    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));

    let (_, call_status) = complete(rust_future);
    assert_eq!(call_status.code, RustCallStatusCode::Cancelled);
}

// Once `free` is called, the inner future should be released, even if wakers still hold a
// reference to the RustFuture
#[test]
fn test_release_future() {
    let (sender, rust_future) = channel();
    // Create a weak reference to the channel to use to check if rust_future has dropped its
    // future.
    let channel_weak = Arc::downgrade(&sender.0);
    drop(sender);
    // Create an extra ref to rust_future, simulating a waker that still holds a reference to
    // it
    let rust_future2 = rust_future.clone();

    // Complete the rust future
    rust_future.free();
    // Even though rust_future is still alive, the channel shouldn't be
    assert!(Arc::strong_count(&rust_future2) > 0);
    assert_eq!(channel_weak.strong_count(), 0);
    assert!(channel_weak.upgrade().is_none());
}

// If `free` is called with a continuation still stored, we should call it them then.
//
// This shouldn't happen in practice, but it seems like good defensive programming
#[test]
fn test_complete_with_stored_continuation() {
    let (_sender, rust_future) = channel();

    let continuation_result = poll(&rust_future);
    rust_future.free();
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));
}

// Test what happens if we see a `wake()` call while we're polling the future.  This can
// happen, for example, with futures that are handled by a tokio thread pool.  We should
// schedule another poll of the future in this case.
#[test]
fn test_wake_during_poll() {
    let mut first_time = true;
    let future = std::future::poll_fn(move |ctx| {
        if first_time {
            first_time = false;
            // Wake the future while we are in the middle of polling it
            ctx.waker().wake_by_ref();
            Poll::Pending
        } else {
            // The second time we're polled, we're ready
            Poll::Ready(Ok("All done".to_owned()))
        }
    });
    let rust_future: Arc<RustFuture<RustBuffer>> =
        Arc::new(RustFuture::new(Box::pin(future), crate::UniFfiTag));
    let continuation_result = poll(&rust_future);
    // The continuation function should called immediately
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Wake));
    // A second poll should finish the future
    let continuation_result = poll(&rust_future);
    assert_eq!(continuation_result.get(), Some(&RustFuturePoll::Ready));
    let (return_buf, call_status) = complete(rust_future);
    assert_eq!(call_status.code, RustCallStatusCode::Success);
    assert_eq!(
        <String as Lift<crate::UniFfiTag>>::try_lift(return_buf).unwrap(),
        "All done"
    );
}

// === Lock discipline: continuations are never invoked while the scheduler is locked ===
//
// A continuation callback is foreign code that takes foreign locks.  Swift is the worst case:
// resuming a continuation from another thread takes the awaiting task's status-record lock, and
// Swift calls `rust_future_cancel` from a `withTaskCancellationHandler` handler that already holds
// that same lock.  So if the scheduler invoked callbacks with its own lock held, a completing
// future and a cancelling foreign thread would deadlock against each other:
//
//   waker thread:   Scheduler::wake [scheduler lock] -> continuation -> [foreign lock] BLOCKED
//   foreign thread: cancel handler  [foreign lock]   -> cancel       -> [scheduler lock] BLOCKED
//
// `Scheduler` avoids this by taking the callback out of its state under the lock, releasing the
// lock, and only then calling foreign code.  These tests drive that interleaving so the property
// stays true.

thread_local! {
    /// Whether this thread is inside [ForeignTask::with_lock_held].
    static HOLDS_FOREIGN_LOCK: Cell<bool> = const { Cell::new(false) };
}

/// Stands in for the foreign task awaiting a `RustFuture`.
struct ForeignTask {
    /// The foreign runtime lock that resuming the continuation needs (Swift's per-task
    /// status-record lock, Ruby's GVL, ...).
    ///
    /// Modelled as re-entrant, because Swift's is: `withStatusRecordLock` detects that the current
    /// thread already holds the lock and proceeds, which is what lets a cancellation handler
    /// resume its own task's continuation inline.  Another thread blocks.
    lock: Mutex<()>,
    /// Set from inside the continuation, just before it tries to take `lock`.
    in_callback: AtomicBool,
    /// How many times the continuation was invoked.  Must always end up at exactly 1.
    invocations: AtomicUsize,
    /// The poll result the continuation was invoked with.
    result: OnceCell<RustFuturePoll>,
    /// If set, the continuation re-enters the scheduler by cancelling this future.
    reenter: Mutex<Option<Arc<RustFuture<RustBuffer>>>>,
}

impl ForeignTask {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            lock: Mutex::new(()),
            in_callback: AtomicBool::new(false),
            invocations: AtomicUsize::new(0),
            result: OnceCell::new(),
            reenter: Mutex::new(None),
        })
    }

    /// Poll `rust_future`, registering this task's continuation.
    fn poll(self: &Arc<Self>, rust_future: &Arc<RustFuture<RustBuffer>>) {
        let data = Arc::into_raw(Arc::clone(self)) as u64;
        Arc::clone(rust_future).poll(RustFutureContinuationBoundCallback {
            callback: foreign_continuation,
            data,
        });
    }

    fn spin_until_in_callback(&self) {
        while !self.in_callback.load(Ordering::SeqCst) {
            std::hint::spin_loop();
        }
    }

    /// Run `f` holding the foreign runtime lock, the way Swift runs a cancellation handler.
    fn with_lock_held<R>(&self, f: impl FnOnce() -> R) -> R {
        let guard = self.lock.lock().unwrap();
        HOLDS_FOREIGN_LOCK.with(|held| held.set(true));
        let result = f();
        HOLDS_FOREIGN_LOCK.with(|held| held.set(false));
        drop(guard);
        result
    }
}

extern "C" fn foreign_continuation(data: u64, code: RustFuturePoll) {
    let task = unsafe { Arc::from_raw(data as *const ForeignTask) };
    task.invocations.fetch_add(1, Ordering::SeqCst);
    if let Some(rust_future) = task.reenter.lock().unwrap().take() {
        // Re-entering the scheduler from inside a continuation only works if the scheduler
        // released its lock before calling us.
        rust_future.cancel();
    }
    task.in_callback.store(true, Ordering::SeqCst);
    // Resuming the continuation takes the foreign runtime lock -- unless this thread already holds
    // it, which is the re-entrant case Swift allows.
    let _guard = (!HOLDS_FOREIGN_LOCK.with(Cell::get)).then(|| task.lock.lock().unwrap());
    task.result.set(code).expect("continuation invoked twice");
}

/// Run `body` on its own thread and fail if it doesn't finish in time, so that a lock-discipline
/// regression reports as a failure instead of hanging the test run forever.
fn run_with_watchdog(name: &str, timeout: Duration, body: impl FnOnce() + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        body();
        // Ignore send errors: the receiver is gone if we already timed out.
        let _ = tx.send(());
    });
    if rx.recv_timeout(timeout).is_err() {
        panic!("{name} deadlocked (no completion within {timeout:?})");
    }
}

// The future completes on a worker thread while the foreign side cancels from a thread that holds
// the lock the continuation needs.  The handshake makes the interleaving deterministic rather than
// hoping to hit it.
#[test]
fn test_cancel_while_waking_does_not_deadlock() {
    run_with_watchdog(
        "test_cancel_while_waking_does_not_deadlock",
        Duration::from_secs(10),
        || {
            let (sender, rust_future) = channel();
            let task = ForeignTask::new();
            task.poll(&rust_future);

            // Hold the foreign lock, as Swift does across a whole `swift_task_cancel`, and cancel
            // from inside it.
            let waker_thread = task.with_lock_held(|| {
                // Complete the future from a worker thread.  This wakes the RustFuture, which
                // invokes the continuation, which blocks on the foreign lock we hold.
                let waker_thread = thread::spawn(move || sender.send(Ok("All done".into())));
                task.spin_until_in_callback();

                // The continuation is now blocked on us.  Cancelling must not need anything the
                // blocked continuation is holding.
                rust_future.cancel();
                waker_thread
            });
            waker_thread.join().unwrap();

            // The wake won the race, so the continuation was resumed exactly once, with `Wake`.
            assert_eq!(task.invocations.load(Ordering::SeqCst), 1);
            assert_eq!(task.result.get(), Some(&RustFuturePoll::Wake));
            assert!(rust_future.is_cancelled());

            // The foreign side polls again after a `Wake`, and now sees the cancellation.
            let task2 = ForeignTask::new();
            task2.poll(&rust_future);
            assert_eq!(task2.invocations.load(Ordering::SeqCst), 1);
            assert_eq!(task2.result.get(), Some(&RustFuturePoll::Ready));
            let (_, call_status) = complete(rust_future);
            assert_eq!(call_status.code, RustCallStatusCode::Cancelled);
        },
    );
}

// Same AB-BA, run unsynchronized many times over: completion and cancellation race freely, and
// half the iterations use a future that is already complete before the first poll (the "instant
// local read" shape that makes the window easy to hit).  Whichever side wins, the continuation is
// resumed exactly once.
#[test]
fn test_cancel_racing_completion_stress() {
    const ITERATIONS: usize = 5_000;
    run_with_watchdog(
        "test_cancel_racing_completion_stress",
        Duration::from_secs(120),
        || {
            for i in 0..ITERATIONS {
                let (sender, rust_future) = channel();
                let task = ForeignTask::new();
                let completes_immediately = i % 2 == 0;
                if completes_immediately {
                    sender.send(Ok("All done".into()));
                }

                let canceller = {
                    let rust_future = Arc::clone(&rust_future);
                    let task = Arc::clone(&task);
                    thread::spawn(move || task.with_lock_held(|| rust_future.cancel()))
                };
                task.poll(&rust_future);
                let waker_thread = (!completes_immediately)
                    .then(|| thread::spawn(move || sender.send(Ok("All done".into()))));

                canceller.join().unwrap();
                if let Some(waker_thread) = waker_thread {
                    waker_thread.join().unwrap();
                }

                assert_eq!(
                    task.invocations.load(Ordering::SeqCst),
                    1,
                    "iteration {i}: continuation not resumed exactly once"
                );
                assert!(task.result.get().is_some(), "iteration {i}");
                assert!(rust_future.is_cancelled(), "iteration {i}");
            }
        },
    );
}

// A continuation is free to call back into the scheduler -- Swift's continuation resumption can
// synchronously run the awaiting task, which may cancel or free the future.
#[test]
fn test_continuation_can_reenter_scheduler() {
    run_with_watchdog(
        "test_continuation_can_reenter_scheduler",
        Duration::from_secs(10),
        || {
            let (sender, rust_future) = channel();
            let task = ForeignTask::new();
            *task.reenter.lock().unwrap() = Some(Arc::clone(&rust_future));
            task.poll(&rust_future);
            // Wake the future: the continuation runs and cancels from inside the callback.
            sender.wake();
            assert_eq!(task.invocations.load(Ordering::SeqCst), 1);
            assert_eq!(task.result.get(), Some(&RustFuturePoll::Wake));
            assert!(rust_future.is_cancelled());
        },
    );
}
