/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/
 */

mod timer;

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use self::timer::{TimerFuture, TimerService};

/// This use features provided by js_sys and web_sys.
///
/// If we're able to compile this with wasm32-unknown-unknown, then
/// wasm32 futures will be available to the host language (likely Javascript).
#[uniffi::export]
pub async fn wait_with_timer(_old: String, _new: String) {
    TimerFuture::sleep(Duration::from_millis(200)).await;
}

// We also want to allow objects which has state which might rely on such Futures.
#[cfg(not(target_arch = "wasm32"))]
type EventHandlerFut = Pin<Box<dyn Future<Output = ()> + Send>>;
#[cfg(target_arch = "wasm32")]
type EventHandlerFut = Pin<Box<dyn Future<Output = ()>>>;

#[cfg(not(target_arch = "wasm32"))]
type EventHandlerFn = dyn Fn(String, String) -> EventHandlerFut + Send + Sync;
#[cfg(target_arch = "wasm32")]
type EventHandlerFn = dyn Fn(String, String) -> EventHandlerFut;

// Here we have a struct which contains async functions.
// WASM focused bindgen implementations should run this, but it should at least
// compile here.
#[derive(uniffi::Object)]
pub struct SimpleObject {
    inner: Mutex<String>,
    callbacks: Vec<Box<EventHandlerFn>>,
}

impl SimpleObject {
    fn new_with_callback(cb: Box<EventHandlerFn>) -> Arc<Self> {
        Arc::new(SimpleObject {
            inner: Mutex::new("key".to_string()),
            callbacks: vec![cb],
        })
    }
}

#[uniffi::export]
impl SimpleObject {
    pub async fn update(self: Arc<Self>, updated: String) {
        let old = {
            let mut data = self.inner.lock().unwrap();
            let old = data.clone();
            *data = updated.clone();
            old
        };
        for callback in self.callbacks.iter() {
            callback(old.clone(), updated.clone()).await;
        }
    }
}

fn from_static() -> Box<EventHandlerFn> {
    Box::new(|old, new| Box::pin(wait_with_timer(old, new)))
}

// Make an object, with a callback implemented from a static function.
// This relies on a timer, which is implemented for wasm using gloo.
// This is not Send, so EventHandlerFn and EventHandlerFut are different
// for wasm.
#[uniffi::export]
async fn make_object() -> Arc<SimpleObject> {
    SimpleObject::new_with_callback(from_static())
}

uniffi::setup_scaffolding!();
