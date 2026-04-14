/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(uniffi::Record)]
pub struct RecWithDefault {
    #[uniffi(default = 42)]
    pub n: u8,
    #[uniffi(default)]
    pub v: Vec<u32>,
}

#[derive(Default, uniffi::Enum)]
pub enum EnumWithDefault {
    #[default]
    DefaultVariant,
    OtherVariant {
        #[uniffi(default = "default")]
        a: String,
    },
}

#[uniffi::export(default(arg = "DEFAULT"))]
pub fn func_with_default(arg: String) -> String {
    arg
}

#[derive(uniffi::Object, Default)]
pub struct InterfaceWithDefaults;

#[uniffi::export]
impl InterfaceWithDefaults {
    #[uniffi::constructor()]
    pub fn new() -> Self {
        Self
    }

    #[uniffi::method(default(arg = "DEFAULT"))]
    pub fn method_with_default(&self, arg: String) -> String {
        arg
    }
}
