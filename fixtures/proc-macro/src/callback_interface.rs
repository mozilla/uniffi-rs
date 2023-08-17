/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{BasicError, Object};

#[uniffi::export(callback_interface)]
pub trait TestCallbackInterface {
    fn do_nothing(&self);
    fn add(&self, a: u32, b: u32) -> u32;
    fn optional(&self, a: Option<u32>) -> u32;
    fn try_parse_int(&self, value: String) -> Result<u32, BasicError>;
    fn callback_handler(&self, h: std::sync::Arc<Object>) -> u32;
}
