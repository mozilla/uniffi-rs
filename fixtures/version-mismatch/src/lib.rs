/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(not(feature = "proc_macro_v2"))]
#[uniffi::export]
fn a_proc_macro_export() -> u32 {
    1
}

#[cfg(feature = "proc_macro_v2")]
#[uniffi::export]
fn a_proc_macro_export() -> i32 {
    2
}

fn a_udl_function() -> u32 {
    1
}

include!(concat!(env!("OUT_DIR"), "/api_v1.uniffi.rs"));
