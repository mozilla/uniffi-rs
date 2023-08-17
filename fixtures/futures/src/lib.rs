/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

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

            if let Some(waker) = shared_state.waker.take() {
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

/// Async function that is immediatly ready.
#[uniffi::export]
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

// Our error.
#[derive(uniffi::Error, Debug)]
pub enum MyError {
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

/// A megaphone. Be careful with the neighbours.
#[derive(uniffi::Object)]
pub struct Megaphone;

#[uniffi::export]
impl Megaphone {
    /// An async method that yells something after a certain time.
    pub async fn say_after(self: Arc<Self>, ms: u16, who: String) -> String {
        say_after(ms, who).await.to_uppercase()
    }

    // An async method that can throw.
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
#[derive(uniffi::Error, Debug)]
pub enum AsyncError {
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

uniffi::include_scaffolding!("futures");
