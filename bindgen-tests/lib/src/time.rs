/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Test SystemTime and Duration

pub use std::time::{Duration, SystemTime};

#[uniffi::export]
pub fn roundtrip_systemtime(a: SystemTime) -> SystemTime {
    a
}

#[uniffi::export]
pub fn roundtrip_duration(a: Duration) -> Duration {
    a
}
