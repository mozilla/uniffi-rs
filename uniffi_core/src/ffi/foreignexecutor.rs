/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Schedule tasks using a foreign executor.

use std::{
    cell::UnsafeCell,
    future::Future,
    panic,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
};

/// Opaque handle for a foreign task executor.
///
/// Foreign code can either use an actual pointer, or use an integer value casted to it.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ForeignExecutorHandle(pub(crate) *const ());

// Implement Send + Sync for `ForeignExecutor`.  The foreign bindings code is responsible for
// making the `ForeignExecutorCallback` thread-safe.
unsafe impl Send for ForeignExecutorHandle {}

unsafe impl Sync for ForeignExecutorHandle {}

/// Callback to schedule a Rust call with a `ForeignExecutor`. The bindings code registers exactly
/// one of these with the Rust code.
///
/// Delay is an approximate amount of ms to wait before scheduling the call.  Delay is usually 0,
/// which means schedule sometime soon.
///
/// As a special case, when Rust drops the foreign executor, with 'task=null'`.  The foreign
/// bindings should release the reference to the executor that was reserved for Rust.
///
/// This callback can be invoked from any thread, including threads created by Rust.
///
/// The callback should return one of the `ForeignExecutorCallbackResult` values.
pub type ForeignExecutorCallback = extern "C" fn(
    executor: ForeignExecutorHandle,
    delay: u32,
    task: Option<RustTaskCallback>,
    task_data: *const (),
) -> i8;

/// Result code returned by `ForeignExecutorCallback`
#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum ForeignExecutorCallbackResult {
    /// Callback was scheduled successfully
    Success = 0,
    /// Callback couldn't be scheduled because the foreign executor is canceled/closed.
    Cancelled = 1,
    /// Callback couldn't be scheduled because of some other error
    Error = 2,
}

impl ForeignExecutorCallbackResult {
    /// Check the result code for the foreign executor callback
    ///
    /// If the result was `ForeignExecutorCallbackResult.Success`, this method returns `true`.
    ///
    /// If not, this method returns `false`, logging errors for any unexpected return values
    pub fn check_result_code(result: i8) -> bool {
        match result {
            n if n == ForeignExecutorCallbackResult::Success as i8 => true,
            n if n == ForeignExecutorCallbackResult::Cancelled as i8 => false,
            n if n == ForeignExecutorCallbackResult::Error as i8 => {
                log::error!(
                    "ForeignExecutorCallbackResult::Error returned by foreign executor callback"
                );
                false
            }
            n => {
                log::error!("Unknown code ({n}) returned by foreign executor callback");
                false
            }
        }
    }
}

// Option<RustTaskCallback> should use the null pointer optimization and be represented in C as a
// regular pointer.  Let's check that.
static_assertions::assert_eq_size!(usize, Option<RustTaskCallback>);

/// Callback for a Rust task, this is what the foreign executor invokes
///
/// The task will be passed the `task_data` passed to `ForeignExecutorCallback` in addition to one
/// of the `RustTaskCallbackCode` values.
pub type RustTaskCallback = extern "C" fn(*const (), RustTaskCallbackCode);

/// Passed to a `RustTaskCallback` function when the executor invokes them.
///
/// Every `RustTaskCallback` will be invoked eventually, this code is used to distinguish the times
/// when it's invoked successfully vs times when the callback is being called because the foreign
/// executor has been cancelled / shutdown
#[repr(i8)]
#[derive(Debug, PartialEq, Eq)]
pub enum RustTaskCallbackCode {
    /// Successful task callback invocation
    Success = 0,
    /// The `ForeignExecutor` has been cancelled.
    ///
    /// This signals that any progress using the executor should be halted.  In particular, Futures
    /// should not continue to progress.
    Cancelled = 1,
}

static FOREIGN_EXECUTOR_CALLBACK: AtomicUsize = AtomicUsize::new(0);

/// Set the global ForeignExecutorCallback.  This is called by the foreign bindings, normally
/// during initialization.
#[no_mangle]
pub extern "C" fn uniffi_foreign_executor_callback_set(callback: ForeignExecutorCallback) {
    FOREIGN_EXECUTOR_CALLBACK.store(callback as usize, Ordering::Relaxed);
}

fn get_foreign_executor_callback() -> ForeignExecutorCallback {
    match FOREIGN_EXECUTOR_CALLBACK.load(Ordering::Relaxed) {
        0 => panic!("FOREIGN_EXECUTOR_CALLBACK not set"),
        // SAFETY: The below call is okay because we only store values in
        // FOREIGN_EXECUTOR_CALLBACK that were cast from a ForeignExecutorCallback.
        n => unsafe { std::mem::transmute(n) },
    }
}

/// Schedule Rust calls using a foreign executor
#[derive(Debug)]
pub struct ForeignExecutor {
    pub(crate) handle: ForeignExecutorHandle,
}

impl ForeignExecutor {
    pub fn new(executor: ForeignExecutorHandle) -> Self {
        Self { handle: executor }
    }

    /// Schedule a closure to be run.
    ///
    /// This method can be used for "fire-and-forget" style calls, where the calling code doesn't
    /// need to await the result.
    ///
    /// Closure requirements:
    ///   - Send: since the closure will likely run on a different thread
    ///   - 'static: since it runs at an arbitrary time, so all references need to be 'static
    ///   - panic::UnwindSafe: if the closure panics, it should not corrupt any data
    pub fn schedule<F: FnOnce() + Send + 'static + panic::UnwindSafe>(&self, delay: u32, task: F) {
        ScheduledTask::new(task).schedule_callback(self.handle, delay)
    }

    /// Schedule a closure to be run and get a Future for the result
    ///
    /// Closure requirements:
    ///   - Send: since the closure will likely run on a different thread
    ///   - 'static: since it runs at an arbitrary time, so all references need to be 'static
    ///   - panic::UnwindSafe: if the closure panics, it should not corrupt any data
    pub fn run<F: FnOnce() -> T + Send + 'static + panic::UnwindSafe, T>(
        &self,
        delay: u32,
        closure: F,
    ) -> impl Future<Output = T> {
        let future = RunFuture::new(closure);
        future.schedule_callback(self.handle, delay);
        future
    }
}

/// Low-level schedule interface
///
/// When using this function, take care to ensure that the `ForeignExecutor` that holds the
/// `ForeignExecutorHandle` has not been dropped.
///
/// Returns true if the callback was successfully scheduled
pub(crate) fn schedule_raw(
    handle: ForeignExecutorHandle,
    delay: u32,
    callback: RustTaskCallback,
    data: *const (),
) -> bool {
    let result_code = (get_foreign_executor_callback())(handle, delay, Some(callback), data);
    ForeignExecutorCallbackResult::check_result_code(result_code)
}

impl Drop for ForeignExecutor {
    fn drop(&mut self) {
        (get_foreign_executor_callback())(self.handle, 0, None, std::ptr::null());
    }
}
/// Struct that handles the ForeignExecutor::schedule() method
struct ScheduledTask<F> {
    task: F,
}

impl<F> ScheduledTask<F>
where
    F: FnOnce() + Send + 'static + panic::UnwindSafe,
{
    fn new(task: F) -> Self {
        Self { task }
    }

    fn schedule_callback(self, handle: ForeignExecutorHandle, delay: u32) {
        let leaked_ptr: *mut Self = Box::leak(Box::new(self));
        if !schedule_raw(handle, delay, Self::callback, leaked_ptr as *const ()) {
            // If schedule_raw() failed, drop the leaked box since `Self::callback()` has not been
            // scheduled to run.
            unsafe {
                // Note: specifying the Box generic is a good safety measure.  Things would go very
                // bad if Rust inferred the wrong type.
                drop(Box::<Self>::from_raw(leaked_ptr));
            };
        }
    }

    extern "C" fn callback(data: *const (), status_code: RustTaskCallbackCode) {
        // No matter what, we need to call Box::from_raw() to balance the Box::leak() call.
        let scheduled_task = unsafe { Box::from_raw(data as *mut Self) };
        if status_code == RustTaskCallbackCode::Success {
            run_task(scheduled_task.task);
        }
    }
}

/// Struct that handles the ForeignExecutor::run() method
struct RunFuture<T, F> {
    inner: Arc<RunFutureInner<T, F>>,
}

// State inside the RunFuture Arc<>
struct RunFutureInner<T, F> {
    // SAFETY: we only access this once in the scheduled callback
    task: UnsafeCell<Option<F>>,
    mutex: Mutex<RunFutureInner2<T>>,
}

// State inside the RunFuture Mutex<>
struct RunFutureInner2<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

impl<T, F> RunFuture<T, F>
where
    F: FnOnce() -> T + Send + 'static + panic::UnwindSafe,
{
    fn new(task: F) -> Self {
        Self {
            inner: Arc::new(RunFutureInner {
                task: UnsafeCell::new(Some(task)),
                mutex: Mutex::new(RunFutureInner2 {
                    result: None,
                    waker: None,
                }),
            }),
        }
    }

    fn schedule_callback(&self, handle: ForeignExecutorHandle, delay: u32) {
        let raw_ptr = Arc::into_raw(Arc::clone(&self.inner));
        if !schedule_raw(handle, delay, Self::callback, raw_ptr as *const ()) {
            // If `schedule_raw()` failed, make sure to decrement the ref count since
            // `Self::callback()` has not been scheduled to run.
            unsafe {
                // Note: specifying the Arc generic is a good safety measure.  Things would go very
                // bad if Rust inferred the wrong type.
                Arc::<RunFutureInner<T, F>>::decrement_strong_count(raw_ptr);
            };
        }
    }

    extern "C" fn callback(data: *const (), status_code: RustTaskCallbackCode) {
        // No matter what, call `Arc::from_raw()` to balance the `Arc::into_raw()` call in
        // `schedule_callback()`.
        let inner = unsafe { Arc::from_raw(data as *const RunFutureInner<T, F>) };

        // Only drive the future forward on `RustTaskCallbackCode::Success`.
        if status_code == RustTaskCallbackCode::Success {
            let task = unsafe { (*inner.task.get()).take().unwrap() };
            if let Some(result) = run_task(task) {
                let mut inner2 = inner.mutex.lock().unwrap();
                inner2.result = Some(result);
                if let Some(waker) = inner2.waker.take() {
                    waker.wake();
                }
            }
        }
    }
}

impl<T, F> Future for RunFuture<T, F> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<T> {
        let mut inner2 = self.inner.mutex.lock().unwrap();
        match inner2.result.take() {
            Some(v) => Poll::Ready(v),
            None => {
                inner2.waker = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// Run a scheduled task, catching any panics.
///
/// If there are panics, then we will log a warning and return None.
fn run_task<F: FnOnce() -> T + panic::UnwindSafe, T>(task: F) -> Option<T> {
    match panic::catch_unwind(task) {
        Ok(v) => Some(v),
        Err(cause) => {
            let message = if let Some(s) = cause.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = cause.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic!".to_string()
            };
            log::warn!("Error calling UniFFI callback function: {message}");
            None
        }
    }
}

#[cfg(test)]
pub use test::MockEventLoop;

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Once,
    };
    use std::task::Wake;

    /// Simulate an event loop / task queue / coroutine scope on the foreign side
    ///
    /// This simply collects scheduled calls into a Vec for testing purposes.
    ///
    /// Most of the MockEventLoop methods are `pub` since it's also used by the `rustfuture` tests.
    pub struct MockEventLoop {
        // Wrap everything in a mutex since we typically share access to MockEventLoop via an Arc
        inner: Mutex<MockEventLoopInner>,
    }

    pub struct MockEventLoopInner {
        // calls that have been scheduled
        calls: Vec<(u32, Option<RustTaskCallback>, *const ())>,
        // has the event loop been shutdown?
        is_shutdown: bool,
    }

    static FOREIGN_EXECUTOR_CALLBACK_INIT: Once = Once::new();

    impl MockEventLoop {
        pub fn new() -> Arc<Self> {
            // Make sure we install a foreign executor callback that can deal with mock event loops
            FOREIGN_EXECUTOR_CALLBACK_INIT
                .call_once(|| uniffi_foreign_executor_callback_set(mock_executor_callback));

            Arc::new(Self {
                inner: Mutex::new(MockEventLoopInner {
                    calls: vec![],
                    is_shutdown: false,
                }),
            })
        }

        /// Create a new ForeignExecutorHandle
        pub fn new_handle(self: &Arc<Self>) -> ForeignExecutorHandle {
            // To keep the memory management simple, we simply leak an arc reference for this.  We
            // only create a handful of these in the tests so there's no need for proper cleanup.
            ForeignExecutorHandle(Arc::into_raw(Arc::clone(self)) as *const ())
        }

        pub fn new_executor(self: &Arc<Self>) -> ForeignExecutor {
            ForeignExecutor {
                handle: self.new_handle(),
            }
        }

        /// Get the current number of scheduled calls
        pub fn call_count(&self) -> usize {
            self.inner.lock().unwrap().calls.len()
        }

        /// Get the last scheduled call
        pub fn last_call(&self) -> (u32, Option<RustTaskCallback>, *const ()) {
            self.inner
                .lock()
                .unwrap()
                .calls
                .last()
                .cloned()
                .expect("no calls scheduled")
        }

        /// Run all currently scheduled calls
        pub fn run_all_calls(&self) {
            let mut inner = self.inner.lock().unwrap();
            let is_shutdown = inner.is_shutdown;
            for (_delay, callback, data) in inner.calls.drain(..) {
                if !is_shutdown {
                    callback.unwrap()(data, RustTaskCallbackCode::Success);
                } else {
                    callback.unwrap()(data, RustTaskCallbackCode::Cancelled);
                }
            }
        }

        /// Shutdown the eventloop, causing scheduled calls and future calls to be cancelled
        pub fn shutdown(&self) {
            self.inner.lock().unwrap().is_shutdown = true;
        }
    }

    // `ForeignExecutorCallback` that we install for testing
    extern "C" fn mock_executor_callback(
        handle: ForeignExecutorHandle,
        delay: u32,
        task: Option<RustTaskCallback>,
        task_data: *const (),
    ) -> i8 {
        let eventloop = handle.0 as *const MockEventLoop;
        let mut inner = unsafe { (*eventloop).inner.lock().unwrap() };
        if inner.is_shutdown {
            ForeignExecutorCallbackResult::Cancelled as i8
        } else {
            inner.calls.push((delay, task, task_data));
            ForeignExecutorCallbackResult::Success as i8
        }
    }

    #[test]
    fn test_schedule_raw() {
        extern "C" fn callback(data: *const (), _status_code: RustTaskCallbackCode) {
            unsafe {
                *(data as *mut u32) += 1;
            }
        }

        let eventloop = MockEventLoop::new();

        let value: u32 = 0;
        assert_eq!(eventloop.call_count(), 0);

        schedule_raw(
            eventloop.new_handle(),
            0,
            callback,
            &value as *const u32 as *const (),
        );
        assert_eq!(eventloop.call_count(), 1);
        assert_eq!(value, 0);

        eventloop.run_all_calls();
        assert_eq!(eventloop.call_count(), 0);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_schedule() {
        let eventloop = MockEventLoop::new();
        let executor = eventloop.new_executor();
        let value = Arc::new(AtomicU32::new(0));
        assert_eq!(eventloop.call_count(), 0);

        let value2 = value.clone();
        executor.schedule(0, move || {
            value2.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(eventloop.call_count(), 1);
        assert_eq!(value.load(Ordering::Relaxed), 0);

        eventloop.run_all_calls();
        assert_eq!(eventloop.call_count(), 0);
        assert_eq!(value.load(Ordering::Relaxed), 1);
    }

    #[derive(Default)]
    struct MockWaker {
        wake_count: AtomicU32,
    }

    impl Wake for MockWaker {
        fn wake(self: Arc<Self>) {
            self.wake_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_run() {
        let eventloop = MockEventLoop::new();
        let executor = eventloop.new_executor();
        let mock_waker = Arc::new(MockWaker::default());
        let waker = Waker::from(mock_waker.clone());
        let mut context = Context::from_waker(&waker);
        assert_eq!(eventloop.call_count(), 0);

        let mut future = executor.run(0, move || "test-return-value");
        assert_eq!(eventloop.call_count(), 1);
        assert_eq!(Pin::new(&mut future).poll(&mut context), Poll::Pending);
        assert_eq!(mock_waker.wake_count.load(Ordering::Relaxed), 0);

        eventloop.run_all_calls();
        assert_eq!(eventloop.call_count(), 0);
        assert_eq!(mock_waker.wake_count.load(Ordering::Relaxed), 1);
        assert_eq!(
            Pin::new(&mut future).poll(&mut context),
            Poll::Ready("test-return-value")
        );
    }

    #[test]
    fn test_drop() {
        let eventloop = MockEventLoop::new();
        let executor = eventloop.new_executor();

        drop(executor);
        // Calling drop should schedule a call with null task data.
        assert_eq!(eventloop.call_count(), 1);
        assert_eq!(eventloop.last_call().1, None);
    }

    // Test that cancelled calls never run
    #[test]
    fn test_cancelled_call() {
        let eventloop = MockEventLoop::new();
        let executor = eventloop.new_executor();
        // Create a shared counter
        let counter = Arc::new(AtomicU32::new(0));
        // schedule increments using both `schedule()` and run()`
        let counter_clone = Arc::clone(&counter);
        executor.schedule(0, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        let counter_clone = Arc::clone(&counter);
        let future = executor.run(0, move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        // shutdown the eventloop before the scheduled call gets a chance to run.
        eventloop.shutdown();
        // `run_all_calls()` will cause the scheduled task callbacks to run, but will pass
        // `RustTaskCallbackCode::Cancelled` to it.  This drop the scheduled closure without executing
        // it.
        eventloop.run_all_calls();

        assert_eq!(counter.load(Ordering::Relaxed), 0);
        drop(future);
    }

    // Test that when scheduled calls are cancelled, the closures are dropped properly
    #[test]
    fn test_cancellation_drops_closures() {
        let eventloop = MockEventLoop::new();
        let executor = eventloop.new_executor();

        // Create an Arc<> that we will move into the closures to test if they are dropped or not
        let arc = Arc::new(0);
        let arc_clone = Arc::clone(&arc);
        executor.schedule(0, move || assert_eq!(*arc_clone, 0));
        let arc_clone = Arc::clone(&arc);
        let future = executor.run(0, move || assert_eq!(*arc_clone, 0));

        // shutdown the eventloop and run the (cancelled) scheduled calls.
        eventloop.shutdown();
        eventloop.run_all_calls();
        // try to schedule some more calls now that the loop has been shutdown
        let arc_clone = Arc::clone(&arc);
        executor.schedule(0, move || assert_eq!(*arc_clone, 0));
        let arc_clone = Arc::clone(&arc);
        let future2 = executor.run(0, move || assert_eq!(*arc_clone, 0));

        // Drop the futures so they don't hold on to any references
        drop(future);
        drop(future2);

        // All of these closures should have been dropped by now, there only remaining arc
        // reference should be the original
        assert_eq!(Arc::strong_count(&arc), 1);
    }
}
