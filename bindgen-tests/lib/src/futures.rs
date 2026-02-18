/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Async functions that round-trip data.
//!
//! The purpose of these is to test the lift/lower implementations in the async code.

use crate::callback_interfaces::CallbackInterfaceNumbers;
use crate::errors::TestError;
use std::{collections::HashMap, sync::Arc};

#[uniffi::export]
async fn async_roundtrip_u8(v: u8) -> u8 {
    v
}

#[uniffi::export]
async fn async_roundtrip_i8(v: i8) -> i8 {
    v
}

#[uniffi::export]
async fn async_roundtrip_u16(v: u16) -> u16 {
    v
}

#[uniffi::export]
async fn async_roundtrip_i16(v: i16) -> i16 {
    v
}

#[uniffi::export]
async fn async_roundtrip_u32(v: u32) -> u32 {
    v
}

#[uniffi::export]
async fn async_roundtrip_i32(v: i32) -> i32 {
    v
}

#[uniffi::export]
async fn async_roundtrip_u64(v: u64) -> u64 {
    v
}

#[uniffi::export]
async fn async_roundtrip_i64(v: i64) -> i64 {
    v
}

#[uniffi::export]
async fn async_roundtrip_f32(v: f32) -> f32 {
    v
}

#[uniffi::export]
async fn async_roundtrip_f64(v: f64) -> f64 {
    v
}

#[uniffi::export]
async fn async_roundtrip_string(v: String) -> String {
    v
}

#[uniffi::export]
async fn async_roundtrip_vec(v: Vec<u32>) -> Vec<u32> {
    v
}

#[uniffi::export]
async fn async_roundtrip_map(v: HashMap<String, String>) -> HashMap<String, String> {
    v
}

#[uniffi::export]
async fn async_throw_error() -> Result<(), TestError> {
    Err(TestError::Failure1)
}

#[derive(uniffi::Object)]
struct AsyncInterface {
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

#[uniffi::export]
async fn async_roundtrip_obj(v: Arc<AsyncInterface>) -> Arc<AsyncInterface> {
    v
}

/// Async version of `TestCallbackInterface`
#[uniffi::export(callback_interface)]
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
async fn invoke_test_async_callback_interface_noop(cbi: Box<dyn TestAsyncCallbackInterface>) {
    cbi.noop().await
}

#[uniffi::export]
async fn invoke_test_async_callback_interface_get_value(
    cbi: Box<dyn TestAsyncCallbackInterface>,
) -> u32 {
    cbi.get_value().await
}

#[uniffi::export]
async fn invoke_test_async_callback_interface_set_value(
    cbi: Box<dyn TestAsyncCallbackInterface>,
    value: u32,
) {
    cbi.set_value(value).await
}

#[uniffi::export]
async fn invoke_test_async_callback_interface_throw_if_equal(
    cbi: Box<dyn TestAsyncCallbackInterface>,
    numbers: CallbackInterfaceNumbers,
) -> Result<CallbackInterfaceNumbers, TestError> {
    cbi.throw_if_equal(numbers).await
}
