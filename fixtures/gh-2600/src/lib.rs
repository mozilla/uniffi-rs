/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicU32, Ordering};

uniffi::setup_scaffolding!("gh_2600");

static DROP_COUNT: AtomicU32 = AtomicU32::new(0);

#[uniffi::export]
fn drop_count() -> u32 {
    DROP_COUNT.load(Ordering::Relaxed)
}

#[derive(Default, uniffi::Object)]
#[allow(unused)]
#[repr(align(32))]
pub struct MyStruct256(u8);

/// This is the problematic struct:
/// it gets dropped before its end of life...
#[uniffi::export]
impl MyStruct256 {
    pub fn method(&self) {}

    #[uniffi::constructor]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Drop for MyStruct256 {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
