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
#[doc(hidden)]
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
#[doc(hidden)]
pub type ForeignExecutorCallback = extern "C" fn(
    executor: ForeignExecutorHandle,
    delay: u32,
    task: Option<RustTaskCallback>,
    task_data: *const (),
);

// Option<RustTaskCallback> should use the null pointer optimization and be represented in C as a
// regular pointer.  Let's check that.
static_assertions::assert_eq_size!(usize, Option<RustTaskCallback>);

/// Callback for a Rust task, this is what the foreign executor invokes
#[doc(hidden)]
pub type RustTaskCallback = extern "C" fn(*const ());

static FOREIGN_EXECUTOR_CALLBACK: AtomicUsize = AtomicUsize::new(0);

/// Set the global ForeignExecutorCallback.  This is called by the foreign bindings, normally
/// during initialization.
#[no_mangle]
#[doc(hidden)]
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
///
/// ForeignExecutors are created by the foreign language and passed into Rust.  Different languages
/// use different classes:
/// - On Kotlin this is a CoroutineScope (e.g. `CoroutineScope(Dispatchers.IO)`)
/// - On Python this is an asyncio.EventLoop (e.g. `asyncio.get_running_loop()`)
/// - On Swift this is a UniFFI defined struct named `UniFfiForeignExecutor` that simply stores a
///   task priority (e.g. `UniFfiForeignExecutor(priority: TaskPriority.background)`)
#[derive(Debug)]
pub struct ForeignExecutor {
    pub(crate) handle: ForeignExecutorHandle,
}

impl ForeignExecutor {
    pub fn new(executor: ForeignExecutorHandle) -> Self {
        Self { handle: executor }
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
}

/// Schedule a closure to be run and get a Future for the result
///
/// ```ignore
/// uniffi::run!(&my_struct.foreign_executor, {
///    // run this code, scheduled by the foreign executor
/// }).await
///
/// uniffi::run!(&my_struct.foreign_executor, 100, {
///    // run this code after a 100ms delay
/// }).await
/// ```
///
/// - The first parameter is a [ForeignExecutor] ref.  Unlike [ForeignExecutor::run], this macro
///   will release the borrow before constructing the move closure.  This supports a major use-case
///   for foreign executors, scheduling a call using a foreign executor stored inside a UniFFI
///   interface (See the [futures example](https://github.com/mozilla/uniffi-rs/blob/main/examples/futures/src/lib.rs).
/// - The second, optional, parameter is a delay is milliseconds
/// - The third parameter is a closure to run, which must be:
///   - Send: since the closure will likely run on a different thread
///   - 'static: since it runs at an arbitrary time, so all references need to be 'static
///   - panic::UnwindSafe: if the closure panics, it should not corrupt any data
#[macro_export]
macro_rules! run {
    ($executor:expr, $closure:expr) => {
        $crate::run!($executor, 0, $closure)
    };

    ($executor:expr, $delay:expr, $closure:expr) => {{
        use std::any::{Any, TypeId};
        use std::borrow::Borrow;
        use $crate::ForeignExecutor;
        assert_eq!(
            $executor.type_id(),
            TypeId::of::<ForeignExecutor>(),
            "First argument is not a &uniffi::ForeignExecutor"
        );
        unsafe {
            // Use borrow() to try to coerce $executor to a ForeignExecutor ref.  This allows
            // users to leave out the `&`.
            let executor: &ForeignExecutor = $executor.borrow();
            // Convert the executor to a raw pointer, than back to a reference.  This releases
            // the borrow as described in the docstring.  This is only safe if we're sure that
            // the underlying owned instance still exists, but this is a safe assumption:
            //   - Note that the only code that will run in this thread is the
            //     `executor.run()` call.  You can't implement Drop for references, so there's
            //     no chance that dropping the reference will cause some other code to run.
            //   - The owned value won't be dropped in this thread by the `executor.run()`
            //     call.
            //   - The owned value won't be dropped by another thread.  We currently have a
            //     shared reference, so no other thread should be able to create a mutable
            //     reference.
            let executor = &*(executor as *const ForeignExecutor);
            executor.run($delay, $closure)
        }
    }};
}

/// Schedule a closure to be run, without getting a Future
///
/// This can be used for "fire-and-forget" style calls, where the calling code doesn't
/// need to await the result.
///
/// ```ignore
/// uniffi::schedule!(&my_struct.foreign_executor, {
///    // run this code, scheduled by the foreign executor
/// })
///
/// uniffi::schedule!(&my_struct.foreign_executor, 100, {
///    // run this code after a 100ms delay
/// })
/// ```
///
/// - The first parameter is a [ForeignExecutor] ref.  Unlike [ForeignExecutor::schedule], this macro
///   will release the borrow before constructing the move closure.  This supports a major use-case
///   for foreign executors, scheduling a call using a foreign executor stored inside a UniFFI
///   interface (See the [futures example](https://github.com/mozilla/uniffi-rs/blob/main/examples/futures/src/lib.rs).
/// - The second, optional, parameter is a delay is milliseconds
/// - The third parameter is a closure to run, which must be:
///   - Send: since the closure will likely schedule on a different thread
///   - 'static: since it runs at an arbitrary time, so all references need to be 'static
///   - panic::UnwindSafe: if the closure panics, it should not corrupt any data
#[macro_export]
macro_rules! schedule {
    ($executor:expr, $closure:expr) => {
        $crate::schedule!($executor, 0, $closure)
    };

    ($executor:expr, $delay:expr, $closure:expr) => {{
        use std::any::{Any, TypeId};
        use std::borrow::Borrow;
        use $crate::ForeignExecutor;
        assert_eq!(
            $executor.type_id(),
            TypeId::of::<ForeignExecutor>(),
            "First argument is not a &uniffi::ForeignExecutor"
        );
        unsafe {
            // See the run! macro for a description of what's happening here
            let executor: &ForeignExecutor = $executor.borrow();
            let executor = &*(executor as *const ForeignExecutor);
            executor.schedule($delay, $closure)
        }
    }};
}

/// Low-level schedule interface
///
/// When using this function, take care to ensure that the ForeignExecutor that holds the
/// ForeignExecutorHandle has not been dropped.
pub(crate) fn schedule_raw(
    handle: ForeignExecutorHandle,
    delay: u32,
    callback: RustTaskCallback,
    data: *const (),
) {
    (get_foreign_executor_callback())(handle, delay, Some(callback), data)
}

impl Drop for ForeignExecutor {
    fn drop(&mut self) {
        (get_foreign_executor_callback())(self.handle, 0, None, std::ptr::null())
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
        let leaked_ptr: *const Self = Box::leak(Box::new(self));
        schedule_raw(handle, delay, Self::callback, leaked_ptr as *const ());
    }

    extern "C" fn callback(data: *const ()) {
        run_task(unsafe { Box::from_raw(data as *mut Self).task });
    }
}

/// Struct that handles the ForeignExecutor::run() method
struct RunFuture<T, F> {
    inner: Arc<RunFutureInner<T, F>>,
}

// State inside the RunFuture Arc<>
struct RunFutureInner<T, F> {
    // SAFETY: we only access this once in the scheduled callback
    task: RunFutureTask<F>,
    mutex: Mutex<RunFutureResult<T>>,
}

/// Scheduled task running in a foreign executor
struct RunFutureTask<F>(UnsafeCell<Option<F>>);

impl<F> RunFutureTask<F> {
    fn new(task: F) -> Self {
        Self(UnsafeCell::new(Some(task)))
    }
}

impl<F, T> RunFutureTask<F>
where
    F: FnOnce() -> T + panic::UnwindSafe,
{
    /// Run the scheduled task.
    ///
    /// The result is wrapped in an Option.  If the function panics, it will be logged and None
    /// will be returned.
    ///
    /// SAFETY: Only call this method once
    unsafe fn run(&self) -> Option<T> {
        run_task((*self.0.get()).take().unwrap())
    }
}

/// Manually mark RunFutureTask as Sync.  This is true as long as the safety rules are followed.
/// `run()` will only be called once and therefore the UnsafeCell will only be accessed from one
/// thread.

unsafe impl<F: Send> Sync for RunFutureTask<F> {}

// State inside the RunFuture Mutex<>
struct RunFutureResult<T> {
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
                task: RunFutureTask::new(task),
                mutex: Mutex::new(RunFutureResult {
                    result: None,
                    waker: None,
                }),
            }),
        }
    }

    fn schedule_callback(&self, handle: ForeignExecutorHandle, delay: u32) {
        let raw_ptr = Arc::into_raw(Arc::clone(&self.inner));
        schedule_raw(handle, delay, Self::callback, raw_ptr as *const ());
    }

    extern "C" fn callback(data: *const ()) {
        unsafe {
            let inner = Arc::from_raw(data as *const RunFutureInner<T, F>);
            // Note: it's safe to call `run()` here since we ensure that this callback is only
            // scheduled to run once.
            if let Some(result) = inner.task.run() {
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
pub use test::MockExecutor;

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Once,
    };
    use std::task::Wake;

    static MOCK_EXECUTOR_INIT: Once = Once::new();

    // Executor for testing that stores scheduled calls in a Vec
    pub struct MockExecutor {
        pub calls: &'static Mutex<Vec<(u32, Option<RustTaskCallback>, *const ())>>,
        pub executor: Option<ForeignExecutor>,
    }

    impl MockExecutor {
        pub fn new() -> Self {
            // Create a boxed call list and immediately leak it, this will be our mock executor
            let calls = Box::leak(Box::new(Mutex::new(Vec::new())));
            let executor = ForeignExecutor {
                handle: unsafe { std::mem::transmute(calls as *const _) },
            };
            // Setup a callback to handle our handles
            MOCK_EXECUTOR_INIT
                .call_once(|| uniffi_foreign_executor_callback_set(mock_executor_callback));

            Self {
                calls,
                executor: Some(executor),
            }
        }

        pub fn handle(&self) -> Option<ForeignExecutorHandle> {
            self.executor.as_ref().map(|e| e.handle)
        }

        pub fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }

        pub fn run_all_calls(&self) {
            let mut calls = self.calls.lock().unwrap();
            for (_delay, callback, data) in calls.drain(..) {
                callback.unwrap()(data);
            }
        }

        pub fn schedule_raw(&self, delay: u32, callback: RustTaskCallback, data: *const ()) {
            let handle = self.executor.as_ref().unwrap().handle;
            schedule_raw(handle, delay, callback, data)
        }

        pub fn schedule<F: FnOnce() + Send + panic::UnwindSafe + 'static>(
            &self,
            delay: u32,
            closure: F,
        ) {
            self.executor.as_ref().unwrap().schedule(delay, closure)
        }

        pub fn run<F: FnOnce() -> T + Send + panic::UnwindSafe + 'static, T>(
            &self,
            delay: u32,
            closure: F,
        ) -> impl Future<Output = T> {
            self.executor.as_ref().unwrap().run(delay, closure)
        }

        pub fn drop_executor(&mut self) {
            self.executor = None;
        }
    }

    impl Default for MockExecutor {
        fn default() -> Self {
            Self::new()
        }
    }

    // Mock executor callback pushes calls to a ScheduledCalls
    extern "C" fn mock_executor_callback(
        executor: ForeignExecutorHandle,
        delay: u32,
        task: Option<RustTaskCallback>,
        task_data: *const (),
    ) {
        unsafe {
            let calls: *mut Mutex<Vec<(u32, Option<RustTaskCallback>, *const ())>> =
                std::mem::transmute(executor);
            calls
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .push((delay, task, task_data));
        }
    }

    #[test]
    fn test_schedule_raw() {
        extern "C" fn callback(data: *const ()) {
            unsafe {
                *(data as *mut u32) += 1;
            }
        }

        let executor = MockExecutor::new();

        let value: u32 = 0;
        assert_eq!(executor.call_count(), 0);

        executor.schedule_raw(0, callback, &value as *const u32 as *const ());
        assert_eq!(executor.call_count(), 1);
        assert_eq!(value, 0);

        executor.run_all_calls();
        assert_eq!(executor.call_count(), 0);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_schedule() {
        let executor = MockExecutor::new();
        let value = Arc::new(AtomicU32::new(0));
        assert_eq!(executor.call_count(), 0);

        let value2 = value.clone();
        executor.schedule(0, move || {
            value2.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(executor.call_count(), 1);
        assert_eq!(value.load(Ordering::Relaxed), 0);

        executor.run_all_calls();
        assert_eq!(executor.call_count(), 0);
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
        let executor = MockExecutor::new();
        let mock_waker = Arc::new(MockWaker::default());
        let waker = Waker::from(mock_waker.clone());
        let mut context = Context::from_waker(&waker);
        assert_eq!(executor.call_count(), 0);

        let mut future = executor.run(0, move || "test-return-value");
        assert_eq!(executor.call_count(), 1);
        assert_eq!(Pin::new(&mut future).poll(&mut context), Poll::Pending);
        assert_eq!(mock_waker.wake_count.load(Ordering::Relaxed), 0);

        executor.run_all_calls();
        assert_eq!(executor.call_count(), 0);
        assert_eq!(mock_waker.wake_count.load(Ordering::Relaxed), 1);
        assert_eq!(
            Pin::new(&mut future).poll(&mut context),
            Poll::Ready("test-return-value")
        );
    }

    #[test]
    fn test_drop() {
        let mut executor = MockExecutor::new();

        executor.schedule(0, || {});
        assert_eq!(executor.call_count(), 1);

        executor.drop_executor();
        assert_eq!(executor.call_count(), 2);
        let calls = executor.calls.lock().unwrap();
        let drop_call = calls.last().unwrap();
        assert_eq!(drop_call.1, None);
    }
}
