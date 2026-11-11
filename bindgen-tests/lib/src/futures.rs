/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Async functions that round-trip data.
//!
//! The purpose of these is to test the lift/lower implementations in the async code.

use crate::callback_interfaces::CallbackInterfaceNumbers;
use crate::errors::TestError;
use std::{
    collections::HashMap,
    future::poll_fn,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Condvar, Mutex,
    },
    task::Poll,
    thread,
    time::Duration,
};

#[uniffi::export]
pub async fn async_roundtrip_u8(v: u8) -> u8 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_i8(v: i8) -> i8 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_u16(v: u16) -> u16 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_i16(v: i16) -> i16 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_u32(v: u32) -> u32 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_i32(v: i32) -> i32 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_u64(v: u64) -> u64 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_i64(v: i64) -> i64 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_f32(v: f32) -> f32 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_f64(v: f64) -> f64 {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_string(v: String) -> String {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_vec(v: Vec<u32>) -> Vec<u32> {
    v
}

#[uniffi::export]
pub async fn async_roundtrip_map(v: HashMap<String, String>) -> HashMap<String, String> {
    v
}

#[uniffi::export]
pub async fn async_throw_error() -> Result<(), TestError> {
    Err(TestError::Failure1)
}

#[derive(uniffi::Object)]
pub struct AsyncInterface {
    name: String,
}

#[uniffi::export]
impl AsyncInterface {
    #[uniffi::constructor]
    pub fn new(name: String) -> Arc<Self> {
        Arc::new(AsyncInterface { name })
    }

    pub async fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(uniffi::Record)]
pub struct AsyncRecord {
    pub name: String,
}

#[uniffi::export]
impl AsyncRecord {
    pub async fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[uniffi::export]
pub async fn async_roundtrip_obj(v: Arc<AsyncInterface>) -> Arc<AsyncInterface> {
    v
}

/// Async version of `TestCallbackInterface`
#[uniffi::export(callback_interface)]
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait TestAsyncCallbackInterface: Send + Sync {
    async fn noop(&self);
    async fn get_value(&self) -> u32;
    async fn set_value(&self, value: u32);
    async fn throw_if_equal(
        &self,
        numbers: CallbackInterfaceNumbers,
    ) -> Result<CallbackInterfaceNumbers, TestError>;
}

#[uniffi::export]
pub async fn invoke_test_async_callback_interface_noop(cbi: Box<dyn TestAsyncCallbackInterface>) {
    cbi.noop().await
}

#[uniffi::export]
pub async fn invoke_test_async_callback_interface_get_value(
    cbi: Box<dyn TestAsyncCallbackInterface>,
) -> u32 {
    cbi.get_value().await
}

#[uniffi::export]
pub async fn invoke_test_async_callback_interface_set_value(
    cbi: Box<dyn TestAsyncCallbackInterface>,
    value: u32,
) {
    cbi.set_value(value).await
}

#[uniffi::export]
pub async fn invoke_test_async_callback_interface_throw_if_equal(
    cbi: Box<dyn TestAsyncCallbackInterface>,
    numbers: CallbackInterfaceNumbers,
) -> Result<CallbackInterfaceNumbers, TestError> {
    cbi.throw_if_equal(numbers).await
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
        poll_fn(|cx| {
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

/// Like `invoke_test_async_callback_interface_noop`, but listed in
/// `bindings.swift.async_cancellable`.  Cancelling it drops the callback's future, which should
/// cancel the foreign task that's running the callback.
#[uniffi::export]
pub async fn cancellable_invoke_test_async_callback_interface_noop(
    cbi: Box<dyn TestAsyncCallbackInterface>,
) {
    cbi.noop().await
}
