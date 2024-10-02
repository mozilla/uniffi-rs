/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Record)]
pub struct SomeStruct {
    some_field: u32,
}

#[derive(uniffi::Enum)]
pub enum SomeEnum {
    One,
    Two,
}

#[derive(uniffi::Object)]
pub struct SomeObj;

#[uniffi::export]
impl SomeObj {
    pub fn some_method(&self) {}
}

#[uniffi::export]
pub fn some_func() {}

uniffi::setup_scaffolding!();
