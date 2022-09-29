/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// pub fn r#if(_break: u8) {}

#[allow(non_camel_case_types)]
pub enum r#case {
    r#internal,
}

#[allow(non_camel_case_types)]
pub enum r#for {
    #[allow(non_camel_case_types)]
    r#internal { r#break: u8 },
}

#[allow(non_camel_case_types)]
pub struct r#return {
    class: u8,
    switch: Option<u8>,
}

#[allow(non_camel_case_types)]
pub struct r#break {}

impl r#break {
    pub fn class(&self, _internal: u8) {}
    pub fn internal(&self, _class: Option<u8>) {}
}

#[allow(non_camel_case_types)]
#[derive(Debug, thiserror::Error)]
pub enum class {
    #[error("internal error")]
    internal,
}

#[allow(non_camel_case_types)]
#[derive(Debug, thiserror::Error)]
pub enum func {
    #[error("class?")]
    class { object: u8 },
}

include!(concat!(env!("OUT_DIR"), "/keywords.uniffi.rs"));
