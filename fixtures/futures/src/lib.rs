/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use futures::future::{AbortHandle, Abortable, Aborted};

/// Non-blocking timer future.
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();

        // Let's mimic an event coming from somewhere else, like the system.
        thread::spawn(move || {
            thread::sleep(duration);

            let mut shared_state: MutexGuard<_> = thread_shared_state.lock().unwrap();
            shared_state.completed = true;

            // Release the lock BEFORE waking to avoid deadlocks: waker.wake()
            // may invoke a foreign callback that re-polls this future, which
            // would need to acquire shared_state again.
            let waker = shared_state.waker.take();
            drop(shared_state);

            if let Some(waker) = waker {
                waker.wake();
            }
        });

        Self { shared_state }
    }
}

/// Sync function.
#[uniffi::export]
pub fn greet(who: String) -> String {
    format!("Hello, {who}")
}

/// Async function that is immediately ready.
///
/// (This one is defined in the UDL to test UDL support)
pub async fn always_ready() -> bool {
    true
}

#[uniffi::export]
pub async fn void() {}

/// Async function that says something after 2s.
#[uniffi::export]
pub async fn say() -> String {
    TimerFuture::new(Duration::from_secs(2)).await;

    "Hello, Future!".to_string()
}

/// Async function that says something after a certain time.
#[uniffi::export]
pub async fn say_after(ms: u16, who: String) -> String {
    TimerFuture::new(Duration::from_millis(ms.into())).await;

    format!("Hello, {who}!")
}

/// Async function that sleeps!
#[uniffi::export]
pub async fn sleep(ms: u16) -> bool {
    TimerFuture::new(Duration::from_millis(ms.into())).await;

    true
}

/// Async function that sleeps with no return type
#[uniffi::export]
pub async fn sleep_no_return(ms: u16) {
    TimerFuture::new(Duration::from_millis(ms.into())).await;
}

// Our error.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum MyError {
    #[error("Foo")]
    Foo,
}

// An async function that can throw.
#[uniffi::export]
pub async fn fallible_me(do_fail: bool) -> Result<u8, MyError> {
    if do_fail {
        Err(MyError::Foo)
    } else {
        Ok(42)
    }
}

// An async function returning a struct that can throw.
#[uniffi::export]
pub async fn fallible_struct(do_fail: bool) -> Result<Arc<Megaphone>, MyError> {
    if do_fail {
        Err(MyError::Foo)
    } else {
        Ok(new_megaphone())
    }
}

/// Sync function that generates a new `Megaphone`.
///
/// It builds a `Megaphone` which has async methods on it.
#[uniffi::export]
pub fn new_megaphone() -> Arc<Megaphone> {
    Arc::new(Megaphone)
}

/// Async function that generates a new `Megaphone`.
#[uniffi::export]
pub async fn async_new_megaphone() -> Arc<Megaphone> {
    new_megaphone()
}

/// Async function that possibly generates a new `Megaphone`.
#[uniffi::export]
pub async fn async_maybe_new_megaphone(y: bool) -> Option<Arc<Megaphone>> {
    if y {
        Some(new_megaphone())
    } else {
        None
    }
}

/// Async function that inputs `Megaphone`.
#[uniffi::export]
pub async fn say_after_with_megaphone(megaphone: Arc<Megaphone>, ms: u16, who: String) -> String {
    megaphone.say_after(ms, who).await
}

/// A megaphone. Be careful with the neighbours.
#[derive(uniffi::Object)]
pub struct Megaphone;

#[uniffi::export]
impl Megaphone {
    // the default constructor - many bindings will not support this.
    #[uniffi::constructor]
    pub async fn new() -> Arc<Self> {
        TimerFuture::new(Duration::from_millis(0)).await;
        Arc::new(Self {})
    }

    // most should support this.
    #[uniffi::constructor]
    pub async fn secondary() -> Arc<Self> {
        TimerFuture::new(Duration::from_millis(0)).await;
        Arc::new(Self {})
    }

    /// An async method that yells something after a certain time.
    pub async fn say_after(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after(ms, who).await.to_uppercase()
    }

    /// An async method without any extra arguments.
    pub async fn silence(&self) -> String {
        String::new()
    }

    /// An async method that can throw.
    pub async fn fallible_me(self: Arc<Self>, do_fail: bool) -> Result<u8, MyError> {
        if do_fail {
            Err(MyError::Foo)
        } else {
            Ok(42)
        }
    }
}

// The async_runtime attribute used to error when *any* function in the impl block was not async,
// now it should work as long as at least one function *is* async.
#[uniffi::export(async_runtime = "tokio")]
impl Megaphone {
    /// A sync method that yells something immediately.
    pub fn say_now(&self, who: String) -> String {
        format!("Hello, {who}!").to_uppercase()
    }

    /// An async method that yells something after a certain time.
    ///
    /// Uses tokio's timer functionality.
    pub async fn say_after_with_tokio(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after_with_tokio(ms, who).await.to_uppercase()
    }
}

// Say something after a certain amount of time, by using `tokio::time::sleep`
// instead of our own `TimerFuture`.
#[uniffi::export(async_runtime = "tokio")]
pub async fn say_after_with_tokio(ms: u16, who: String) -> String {
    tokio::time::sleep(Duration::from_millis(ms.into())).await;

    format!("Hello, {who} (with Tokio)!")
}

#[derive(uniffi::Object)]
pub struct FallibleMegaphone;

#[uniffi::export]
impl FallibleMegaphone {
    // the default constructor - many bindings will not support this.
    #[uniffi::constructor]
    pub async fn new() -> Result<Arc<Self>, MyError> {
        Err(MyError::Foo)
    }
}

pub struct UdlMegaphone;

impl UdlMegaphone {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn secondary() -> Self {
        Self {}
    }

    pub async fn say_after(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after(ms, who).await.to_uppercase()
    }
}

#[derive(uniffi::Record)]
pub struct MyRecord {
    pub a: String,
    pub b: u32,
}

#[uniffi::export]
pub async fn new_my_record(a: String, b: u32) -> MyRecord {
    MyRecord { a, b }
}

/// Non-blocking timer future.
pub struct BrokenTimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for BrokenTimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl BrokenTimerFuture {
    pub fn new(duration: Duration, fail_after: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();

        // Let's mimic an event coming from somewhere else, like the system.
        thread::spawn(move || {
            thread::sleep(duration);

            let mut shared_state: MutexGuard<_> = thread_shared_state.lock().unwrap();
            shared_state.completed = true;

            if let Some(waker) = shared_state.waker.take() {
                // Do not consume `waker`.
                waker.wake_by_ref();

                // And this is the important part. We are going to call
                // `wake()` a second time. That's incorrect, but that's on
                // purpose, to see how foreign languages will react.
                if fail_after.is_zero() {
                    waker.wake();
                } else {
                    thread::spawn(move || {
                        thread::sleep(fail_after);
                        waker.wake();
                    });
                }
            }
        });

        Self { shared_state }
    }
}

/// Async function that sleeps!
#[uniffi::export]
pub async fn broken_sleep(ms: u16, fail_after: u16) {
    BrokenTimerFuture::new(
        Duration::from_millis(ms.into()),
        Duration::from_millis(fail_after.into()),
    )
    .await;
}

#[derive(uniffi::Record)]
pub struct SharedResourceOptions {
    pub release_after_ms: u16,
    pub timeout_ms: u16,
}

// Our error.
#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum AsyncError {
    #[error("Timeout")]
    Timeout,
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn use_shared_resource(options: SharedResourceOptions) -> Result<(), AsyncError> {
    use once_cell::sync::Lazy;
    use tokio::{
        sync::Mutex,
        time::{sleep, timeout},
    };

    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    let _guard = timeout(
        Duration::from_millis(options.timeout_ms.into()),
        MUTEX.lock(),
    )
    .await
    .map_err(|_| {
        println!("Timeout error in use_shared_resource().  The unit test may hang after this");
        AsyncError::Timeout
    })?;

    sleep(Duration::from_millis(options.release_after_ms.into())).await;
    Ok(())
}

// Example of an trait with async methods
#[uniffi::export]
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait SayAfterTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

// Example of async trait defined in the UDL file
#[uniffi::trait_interface]
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait SayAfterUdlTrait: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

struct SayAfterImpl1;

struct SayAfterImpl2;

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterTrait for SayAfterImpl1 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterTrait for SayAfterImpl2 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[uniffi::export]
fn get_say_after_traits() -> Vec<Arc<dyn SayAfterTrait>> {
    vec![Arc::new(SayAfterImpl1), Arc::new(SayAfterImpl2)]
}

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterUdlTrait for SayAfterImpl1 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterUdlTrait for SayAfterImpl2 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after(ms, who).await
    }
}

#[uniffi::export]
fn get_say_after_udl_traits() -> Vec<Arc<dyn SayAfterUdlTrait>> {
    vec![Arc::new(SayAfterImpl1), Arc::new(SayAfterImpl2)]
}

// Example of an async trait whose Rust implementation uses tokio resources.
#[uniffi::export(with_foreign, async_runtime = "tokio")]
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait SayAfterTraitTokio: Send + Sync {
    async fn say_after(&self, ms: u16, who: String) -> String;
}

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterTraitTokio for SayAfterImpl1 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after_with_tokio(ms, who).await
    }
}

#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
impl SayAfterTraitTokio for SayAfterImpl2 {
    async fn say_after(&self, ms: u16, who: String) -> String {
        say_after_with_tokio(ms, who).await
    }
}

#[uniffi::export]
fn get_say_after_tokio_traits() -> Vec<Arc<dyn SayAfterTraitTokio>> {
    vec![Arc::new(SayAfterImpl1), Arc::new(SayAfterImpl2)]
}

// Async callback interface implemented in foreign code
#[uniffi::export(with_foreign)]
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait AsyncParser: Send + Sync {
    // Simple async method
    async fn as_string(&self, delay_ms: i32, value: i32) -> String;
    // Async method that can throw
    async fn try_from_string(&self, delay_ms: i32, value: String) -> Result<i32, ParserError>;
    // Void return, which requires special handling
    async fn delay(&self, delay_ms: i32);
    // Void return that can also throw
    async fn try_delay(&self, delay_ms: String) -> Result<(), ParserError>;
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum ParserError {
    #[error("NotAnInt")]
    NotAnInt,
    #[error("UnexpectedError")]
    UnexpectedError,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for ParserError {
    fn from(_: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::UnexpectedError
    }
}

#[uniffi::export]
async fn as_string_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32, value: i32) -> String {
    obj.as_string(delay_ms, value).await
}

#[uniffi::export]
async fn try_from_string_using_trait(
    obj: Arc<dyn AsyncParser>,
    delay_ms: i32,
    value: String,
) -> Result<i32, ParserError> {
    obj.try_from_string(delay_ms, value).await
}

#[uniffi::export]
async fn delay_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32) {
    obj.delay(delay_ms).await
}

#[uniffi::export]
async fn try_delay_using_trait(
    obj: Arc<dyn AsyncParser>,
    delay_ms: String,
) -> Result<(), ParserError> {
    obj.try_delay(delay_ms).await
}

#[uniffi::export]
async fn cancel_delay_using_trait(obj: Arc<dyn AsyncParser>, delay_ms: i32) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    thread::spawn(move || {
        // Simulate a different thread aborting the process
        thread::sleep(Duration::from_millis(1));
        abort_handle.abort();
    });
    let future = Abortable::new(obj.delay(delay_ms), abort_registration);
    assert_eq!(future.await, Err(Aborted));
}

/// A test object with no (async) methods.
#[derive(uniffi::Object)]
pub struct TestObject;

#[uniffi::export(async_runtime = "tokio")]
impl TestObject {
    #[uniffi::constructor]
    pub async fn new() -> Self {
        Self
    }
}

/// Tests cancelling an async call.
///
/// `run` is listed in `bindings.swift.async_cancellable` in `uniffi.toml`, so cancelling the Swift
/// task that awaits it cancels and drops the Rust future.  Each instance records what happened to
/// its own futures, so tests that use separate instances can't interfere with each other.
#[derive(uniffi::Object)]
pub struct CancellationTester {
    /// Whether the first poll of `run` blocks until `release_poll` is called
    block_first_poll: bool,
    /// How long after its first poll `run` completes.  `None` means only cancellation ends it.
    complete_after: Option<Duration>,
    poll_count: AtomicU32,
    /// Number of `run` futures that have started and not been dropped yet
    live_count: AtomicU32,
    in_poll: AtomicBool,
    poll_released: Mutex<bool>,
    poll_released_cond: Condvar,
}

#[uniffi::export]
impl CancellationTester {
    #[uniffi::constructor]
    pub fn new(block_first_poll: bool, complete_after_ms: Option<u32>) -> Arc<Self> {
        Arc::new(Self {
            block_first_poll,
            complete_after: complete_after_ms.map(|ms| Duration::from_millis(ms.into())),
            poll_count: AtomicU32::new(0),
            live_count: AtomicU32::new(0),
            in_poll: AtomicBool::new(false),
            poll_released: Mutex::new(false),
            poll_released_cond: Condvar::new(),
        })
    }

    /// Runs until it completes or is cancelled.
    ///
    /// It can't fail, but it's cancellable, so Swift declares it `async throws`.
    pub async fn run(&self) -> bool {
        let _live = LiveGuard::new(&self.live_count);
        let completed = Arc::new(AtomicBool::new(false));
        let mut first_poll = true;
        std::future::poll_fn(|cx| {
            self.poll_count.fetch_add(1, Ordering::SeqCst);
            if first_poll {
                first_poll = false;
                if self.block_first_poll {
                    self.block_until_released();
                }
                if let Some(delay) = self.complete_after {
                    let completed = Arc::clone(&completed);
                    let waker = cx.waker().clone();
                    thread::spawn(move || {
                        thread::sleep(delay);
                        completed.store(true, Ordering::SeqCst);
                        waker.wake();
                    });
                }
            }
            if completed.load(Ordering::SeqCst) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        true
    }

    /// Let a first poll that's blocked because of `block_first_poll` return
    pub fn release_poll(&self) {
        *self.poll_released.lock().unwrap() = true;
        self.poll_released_cond.notify_all();
    }

    pub fn poll_count(&self) -> u32 {
        self.poll_count.load(Ordering::SeqCst)
    }

    pub fn live_count(&self) -> u32 {
        self.live_count.load(Ordering::SeqCst)
    }

    pub fn is_in_poll(&self) -> bool {
        self.in_poll.load(Ordering::SeqCst)
    }
}

impl CancellationTester {
    fn block_until_released(&self) {
        self.in_poll.store(true, Ordering::SeqCst);
        // The timeout stops a failing test from blocking the thread forever
        let _ = self
            .poll_released_cond
            .wait_timeout_while(
                self.poll_released.lock().unwrap(),
                Duration::from_secs(10),
                |released| !*released,
            )
            .unwrap();
        self.in_poll.store(false, Ordering::SeqCst);
    }
}

/// Held by a `CancellationTester::run` future while its body runs, to see when it's dropped
struct LiveGuard<'a>(&'a AtomicU32);

impl<'a> LiveGuard<'a> {
    fn new(live_count: &'a AtomicU32) -> Self {
        live_count.fetch_add(1, Ordering::SeqCst);
        Self(live_count)
    }
}

impl Drop for LiveGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

uniffi::include_scaffolding!("futures");
