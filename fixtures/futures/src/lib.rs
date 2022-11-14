/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[uniffi::export]
fn get_string() -> String {
    "I am a string".to_owned()
}

#[uniffi::export]
async fn get_future() -> String {
    println!("Hello Future");
    "I am a future".to_owned()
}

include!(concat!(env!("OUT_DIR"), "/uniffi_futures.uniffi.rs"));
