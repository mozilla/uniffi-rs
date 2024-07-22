/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{collections::HashMap, sync::Arc};

pub fn r#if(_object: u8) {}

#[allow(non_camel_case_types)]
pub struct r#break {}

impl r#break {
    pub fn class(&self, _object: u8) {}
    pub fn object(&self, _class: Option<u8>) {}
}

#[allow(non_camel_case_types)]
trait r#continue {
    fn r#return(&self, _v: r#return) -> r#return {
        unimplemented!()
    }
    fn r#continue(&self) -> Option<Box<dyn r#continue>> {
        unimplemented!()
    }
    fn r#break(&self, _v: Option<Arc<r#break>>) -> HashMap<u8, Arc<r#break>> {
        unimplemented!()
    }
    fn r#while(&self, _v: Vec<r#while>) -> r#while {
        unimplemented!()
    }
    fn class(&self, _v: HashMap<u8, Vec<class>>) -> Option<HashMap<u8, Vec<class>>> {
        unimplemented!()
    }
}

#[uniffi::export]
impl r#continue for r#break {}

#[allow(non_camel_case_types)]
pub struct r#return {
    class: u8,
    object: Option<u8>,
}

#[allow(non_camel_case_types)]
pub struct r#while {
    class: r#return,
    fun: Option<r#return>,
    object: Vec<r#return>,
    r#break: HashMap<u8, r#return>,
}

#[allow(non_camel_case_types)]
pub enum r#false {
    #[allow(non_camel_case_types)]
    r#true { object: u8 },
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum class {
    object,
}

#[allow(non_camel_case_types)]
#[derive(Debug, thiserror::Error)]
pub enum fun {
    #[error("class?")]
    class { object: u8 },
}

// `FooError` is turned into `FooException`, so this enum will end up
// being named just `Exception`. Without special care, this will clash
// with `kotlin.Exception` class.
#[allow(non_camel_case_types)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("class?")]
    class { object: u8 },
}

#[allow(non_camel_case_types)]
pub struct r#else(pub r#return);
uniffi::custom_newtype!(r#else, r#return);

#[uniffi::export]
pub fn get_else(e: r#else) -> r#else {
    e
}

uniffi::include_scaffolding!("keywords");
