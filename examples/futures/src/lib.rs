/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use async_std::future::{pending, timeout};
use std::sync::Arc;
use std::time::Duration;

/// Async function that says something after a certain time.
#[uniffi::export]
pub async fn say_after(ms: u64, who: String) -> String {
    let never = pending::<()>();
    timeout(Duration::from_millis(ms), never).await.unwrap_err();
    format!("Hello, {who}!")
}

/// Simulates an object that performs a IO bound task in the background
#[derive(uniffi::Object)]
pub struct Store {
    background_executor: uniffi::ForeignExecutor,
}

#[uniffi::export]
impl Store {
    #[uniffi::constructor]
    pub fn new(background_executor: uniffi::ForeignExecutor) -> Arc<Self> {
        Arc::new(Self {
            background_executor,
        })
    }

    /// Load an item from disk, using the background executor
    pub async fn load_item(self: Arc<Self>) -> String {
        uniffi::run!(self.background_executor, move || self.do_load_item()).await
    }

    fn do_load_item(&self) -> String {
        "this was loaded from disk".into()
    }
}

uniffi::setup_scaffolding!();
