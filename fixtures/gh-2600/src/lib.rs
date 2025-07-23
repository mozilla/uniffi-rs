/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::atomic::{AtomicU32, Ordering};

uniffi::setup_scaffolding!("gh_2600");

use std::arch::x86_64::{__m256i, _mm256_set1_epi8};

static DROP_COUNT: AtomicU32 = AtomicU32::new(0);

#[uniffi::export]
fn drop_count() -> u32 {
    DROP_COUNT.load(Ordering::Relaxed)
}

#[derive(uniffi::Object)]
#[allow(unused)]
pub struct MyStruct256(__m256i);

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

// Implement `Default` so clippy doesn't complain
impl Default for MyStruct256 {
    fn default() -> Self {
        Self(unsafe { _mm256_set1_epi8(0) })
    }
}

impl Drop for MyStruct256 {
    fn drop(&mut self) {
        DROP_COUNT.fetch_add(1, Ordering::Relaxed);
    }
}
