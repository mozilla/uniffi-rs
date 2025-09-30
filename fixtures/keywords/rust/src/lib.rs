/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A combined Rust + Python keywords test, ensuring we overlap enough so we have
//! good tests for both.

use std::{collections::HashMap, sync::Arc};

#[uniffi::export]
pub fn r#if(_async: u8) {}

#[allow(non_camel_case_types)]
#[derive(uniffi::Object)]
pub struct r#break {}

impl r#break {
    pub fn r#return(&self, v: r#return) -> r#return {
        v
    }
    pub fn r#break(&self, v: HashMap<u8, Arc<r#break>>) -> Option<HashMap<u8, Arc<r#break>>> {
        Some(v)
    }
    pub fn r#continue(&self, _v: Vec<Box<dyn r#continue>>) {}
    pub fn r#yield(&self, _async: u8) {}
    pub fn r#async(&self, _yield: Option<u8>) {}
}

#[allow(non_camel_case_types)]
#[uniffi::export]
pub trait r#continue: Send + Sync {
    fn r#return(&self, _v: r#return) -> r#return;
    fn r#continue(&self) -> Option<Arc<dyn r#continue>>;
    fn r#break(&self, _v: Option<Arc<r#break>>) -> HashMap<u8, Arc<r#break>>;
    fn r#while(&self, _v: Vec<r#while>) -> r#while;
    fn r#yield(&self, _v: HashMap<u8, Vec<r#yield>>) -> Option<HashMap<u8, Vec<r#yield>>>;
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Object)]
pub struct r#in {}

#[uniffi::export]
impl r#continue for r#in {
    fn r#return(&self, _v: r#return) -> r#return {
        unimplemented!()
    }
    fn r#continue(&self) -> Option<Arc<dyn r#continue>> {
        unimplemented!()
    }
    fn r#break(&self, _v: Option<Arc<r#break>>) -> HashMap<u8, Arc<r#break>> {
        unimplemented!()
    }
    fn r#while(&self, _v: Vec<r#while>) -> r#while {
        unimplemented!()
    }
    fn r#yield(&self, _v: HashMap<u8, Vec<r#yield>>) -> Option<HashMap<u8, Vec<r#yield>>> {
        unimplemented!()
    }
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Record, Debug)]
pub struct r#return {
    #[uniffi(default)]
    r#yield: u8,
    #[uniffi(default)]
    r#async: Option<u8>,
    // bool for python/mypy - #2552
    #[uniffi(default)]
    bool: bool,
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Record)]
pub struct r#while {
    r#return: r#return,
    r#yield: Vec<r#yield>,
    r#break: HashMap<u8, Arc<r#break>>,
    r#for: Option<Arc<r#break>>,
    r#async: Vec<r#return>,
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Enum)]
pub enum r#async {
    #[allow(non_camel_case_types)]
    // bool for python/mypy, #2552
    r#as { r#async: u8, bool: bool },
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Enum, Debug)]
pub enum r#yield {
    r#async,
}

#[allow(non_camel_case_types)]
#[derive(uniffi::Error, Debug, thiserror::Error)]
pub enum r#for {
    #[error("return")]
    r#return { r#return: r#return },

    #[error("yield?")]
    r#yield { r#async: u8 },
}

#[allow(non_camel_case_types)]
pub struct r#else(pub r#for);
uniffi::custom_newtype!(r#else, r#for);

#[uniffi::export]
pub fn get_else(e: r#else) -> r#else {
    e
}

uniffi::setup_scaffolding!("keywords_rust");
