/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/
 */

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

#[derive(uniffi::Object)]
pub struct NoopObject;

#[uniffi::export]
impl NoopObject {
    pub async fn noop(self: Arc<Self>) {
        // NOOP
    }
}

/// This will return a Future which is Send + Sync for non-wasm targets, but
/// not for wasm targets.
#[uniffi::export]
pub async fn static_future(_old: String, _new: String) {
    // NOOP
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

impl fmt::Debug for SimpleObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleObject")
    }
}

impl fmt::Display for SimpleObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SimpleObject {
    #[cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
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
    Box::new(|old, new| Box::pin(static_future(old, new)))
}

// Make an object, with a callback implemented from a static function.
// This relies on a timer, which is implemented for wasm using gloo.
// This is not Send, so EventHandlerFn and EventHandlerFut are different
// for wasm.
#[uniffi::export]
async fn make_object() -> Arc<SimpleObject> {
    SimpleObject::new_with_callback(from_static())
}

#[uniffi::export]
async fn throw_object() -> Result<(), Arc<SimpleObject>> {
    let obj = make_object().await;
    Err(obj)
}

#[derive(uniffi::Object, Debug)]
pub struct ErrorObject;

impl fmt::Display for ErrorObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrorObject")
    }
}

#[uniffi::export]
async fn throw_error_object() -> Result<(), Arc<ErrorObject>> {
    let obj = Arc::new(ErrorObject);
    Err(obj)
}

// These use NoopObject.
#[uniffi::export(with_foreign)]
pub trait SimpleNoopTrait: Send + Sync {
    fn return_void(&self);
    fn return_obj(&self, obj: Arc<NoopObject>);
    fn get_obj(&self) -> Arc<NoopObject>;
}

// NoopObject cannot be used in a return position, so
// `get_obj` can't be implemented as an async method.
#[cfg(not(target_arch = "wasm32"))]
#[uniffi::export(with_foreign)]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait AsyncNoopTrait: Send + Sync {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<NoopObject>);
}

#[cfg(target_arch = "wasm32")]
#[uniffi::export(with_foreign)]
#[async_trait(?Send)]
pub trait AsyncNoopTrait {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<NoopObject>);
}

#[uniffi::export(callback_interface)]
pub trait SimpleNoopCbi: Send + Sync {
    fn return_void(&self);
    fn return_obj(&self, obj: Arc<NoopObject>);
    fn get_obj(&self) -> Arc<NoopObject>;
}

// NoopObject cannot be used in a return position, so
// `get_obj` can't be implemented as an async method.
#[cfg(not(target_arch = "wasm32"))]
#[uniffi::export(callback_interface)]
#[async_trait]
pub trait AsyncNoopCbi: Send + Sync {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<NoopObject>);
}

#[cfg(target_arch = "wasm32")]
#[uniffi::export(callback_interface)]
#[async_trait(?Send)]
pub trait AsyncNoopCbi {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<NoopObject>);
}

// Now use SimpleObject, which has the Send/Sync differences.
#[uniffi::export(with_foreign)]
pub trait SimpleTrait: Send + Sync {
    fn return_void(&self);
    fn return_obj(&self, obj: Arc<SimpleObject>);
    fn get_obj(&self) -> Arc<SimpleObject>;
}

#[cfg(not(target_arch = "wasm32"))]
#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AsyncTrait: Send + Sync {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<SimpleObject>);
}

#[cfg(target_arch = "wasm32")]
#[uniffi::export(with_foreign)]
#[async_trait(?Send)]
pub trait AsyncTrait {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<SimpleObject>);
}

#[uniffi::export(callback_interface)]
pub trait SimpleCbi: Send + Sync {
    fn return_void(&self);
    fn return_obj(&self, obj: Arc<SimpleObject>);
    fn get_obj(&self) -> Arc<SimpleObject>;
}

#[cfg(not(target_arch = "wasm32"))]
#[uniffi::export(callback_interface)]
#[async_trait]
pub trait AsyncSimpleCbi: Send + Sync {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<SimpleObject>);
}

#[cfg(target_arch = "wasm32")]
#[uniffi::export(callback_interface)]
#[async_trait(?Send)]
pub trait AsyncSimpleCbi {
    async fn return_void(&self);
    async fn return_obj(&self, obj: Arc<SimpleObject>);
}

uniffi::setup_scaffolding!();
