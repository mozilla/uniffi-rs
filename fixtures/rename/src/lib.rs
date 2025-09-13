/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[uniffi::export(name = "renamed_function")]
pub fn function(record: Record) -> Enum {
    Enum::Record(record)
}

#[derive(uniffi::Record)]
#[uniffi(name = "RenamedRecord")]
pub struct Record {
    item: i32,
}

#[derive(uniffi::Enum)]
#[uniffi(name = "RenamedEnum")]
pub enum Enum {
    VariantA,
    Record(Record),
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
#[uniffi(name = "RenamedError")]
pub enum MyError {
    #[error("Simple error")]
    Simple,
}

#[derive(uniffi::Object)]
#[uniffi(name = "RenamedObject")]
pub struct Object {
    value: i32,
}

#[uniffi::export(name = "RenamedObject")]
impl Object {
    #[uniffi::constructor(name = "renamed_constructor")]
    pub fn new(value: i32) -> Self {
        Object { value }
    }

    #[uniffi::method(name = "renamed_method")]
    pub fn method(&self) -> i32 {
        self.value
    }
}

// Can't rename traits yet, should be possible though, just trickier.
#[uniffi::export]
pub trait Trait: Send + Sync {
    #[uniffi::method(name = "renamed_trait_method")]
    fn trait_method(&self, value: i32) -> i32;
}

struct TraitImpl {
    multiplier: i32,
}

impl Trait for TraitImpl {
    fn trait_method(&self, value: i32) -> i32 {
        value * self.multiplier
    }
}

#[uniffi::export(name = "create_trait_impl")]
pub fn create_trait_impl(multiplier: i32) -> std::sync::Arc<dyn Trait> {
    std::sync::Arc::new(TraitImpl { multiplier })
}

uniffi::setup_scaffolding!();
