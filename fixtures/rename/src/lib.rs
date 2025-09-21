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

/// BINDINGS TESTS
///
/// The way our tests are setup makes it inconvenient to reuse the above,
/// so we duplicate much of the above for testing renaming tests in bindings.
///
/// All types have a "bindings" prefix, the actual bindings will replace with, eg, "python"
#[uniffi::export]
pub fn binding_function(record: Option<BindingRecord>) -> Result<BindingEnum, BindingError> {
    match record {
        Some(r) => Ok(BindingEnum::Record(r)),
        None => Err(BindingError::Simple),
    }
}

#[derive(uniffi::Record)]
pub struct BindingRecord {
    item: i32,
}

#[derive(uniffi::Enum)]
pub enum BindingEnum {
    VariantA,
    Record(BindingRecord),
}

#[derive(uniffi::Enum)]
pub enum BindingEnumWithFields {
    VariantA {
        binding_int: u32,
    },
    Record {
        binding_record: BindingRecord,
        binding_int: u32,
    },
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum BindingError {
    #[error("Simple error")]
    Simple,
}

#[derive(uniffi::Object)]
pub struct BindingObject {
    value: i32,
}

#[uniffi::export]
impl BindingObject {
    #[uniffi::constructor]
    pub fn new(value: i32) -> Self {
        BindingObject { value }
    }

    pub fn method(&self, arg: i32) -> Result<i32, BindingError> {
        Ok(self.value + arg)
    }
}

#[uniffi::export]
pub trait BindingTrait: Send + Sync {
    fn trait_method(&self, value: i32) -> i32;
}

struct BindingTraitImpl {
    multiplier: i32,
}

impl BindingTrait for BindingTraitImpl {
    fn trait_method(&self, value: i32) -> i32 {
        value * self.multiplier
    }
}

#[uniffi::export]
pub fn create_binding_trait_impl(multiplier: i32) -> std::sync::Arc<dyn BindingTrait> {
    std::sync::Arc::new(BindingTraitImpl { multiplier })
}

uniffi::setup_scaffolding!();
