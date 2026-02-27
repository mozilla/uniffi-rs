/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[uniffi::export(name = "renamed_function")]
pub fn function_to_rename(record: RecordToRename) -> EnumToRename {
    EnumToRename::Record(record)
}

#[derive(uniffi::Record)]
#[uniffi(name = "RenamedRecord")]
pub struct RecordToRename {
    item: i32,
}

#[derive(uniffi::Enum)]
#[uniffi(name = "RenamedEnum")]
pub enum EnumToRename {
    #[uniffi(name = "RenamedVariant")]
    VariantA,
    Record(RecordToRename),
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
#[uniffi(name = "RenamedError")]
pub enum ErrorToRename {
    #[error("Simple error")]
    #[uniffi(name = "RenamedErrorVariant")]
    Simple,
}

#[derive(uniffi::Object)]
#[uniffi(name = "RenamedObject")]
pub struct ObjectToRename {
    value: i32,
}

#[uniffi::export(name = "RenamedObject")]
impl ObjectToRename {
    #[uniffi::constructor(name = "renamed_constructor")]
    pub fn new(value: i32) -> Self {
        ObjectToRename { value }
    }

    #[uniffi::method(name = "renamed_method")]
    pub fn method(&self) -> i32 {
        self.value
    }
}

// Can't rename traits yet, should be possible though, just trickier.
#[uniffi::export]
pub trait TraitToRename: Send + Sync {
    #[uniffi::method(name = "renamed_trait_method")]
    fn trait_method(&self, value: i32) -> i32;
}

struct TraitImpl {
    multiplier: i32,
}

impl TraitToRename for TraitImpl {
    fn trait_method(&self, value: i32) -> i32 {
        value * self.multiplier
    }
}

#[uniffi::export(name = "create_trait_impl")]
pub fn create_trait_to_rename_impl(multiplier: i32) -> std::sync::Arc<dyn TraitToRename> {
    std::sync::Arc::new(TraitImpl { multiplier })
}

/// BINDINGS TESTS
///
/// The way our tests are setup makes it inconvenient to reuse the above,
/// so we duplicate much of the above for testing renaming tests in bindings.
///
/// All types have a "bindings" prefix, the actual bindings will replace with, eg, "python"
#[uniffi::export]
pub fn binding_function_to_rename(
    record: Option<BindingRecordToRename>,
) -> Result<BindingEnumToRename, BindingErrorToRename> {
    match record {
        Some(r) => Ok(BindingEnumToRename::Record(r)),
        None => Err(BindingErrorToRename::Simple),
    }
}

#[derive(uniffi::Record)]
pub struct BindingRecordToRename {
    item: i32,
}

#[derive(uniffi::Enum)]
pub enum BindingEnumToRename {
    VariantA,
    Record(BindingRecordToRename),
}

#[derive(uniffi::Enum)]
pub enum BindingEnumWithFieldsToRename {
    VariantA {
        binding_int: u32,
    },
    Record {
        binding_record: BindingRecordToRename,
        binding_int: u32,
    },
}

#[derive(thiserror::Error, uniffi::Error, Debug)]
pub enum BindingErrorToRename {
    #[error("Simple error")]
    Simple,
}

#[derive(uniffi::Object)]
pub struct BindingObjectToRename {
    value: i32,
}

#[uniffi::export]
impl BindingObjectToRename {
    #[uniffi::constructor]
    pub fn new(value: i32) -> Self {
        BindingObjectToRename { value }
    }

    pub fn method(&self, arg: i32) -> Result<i32, BindingErrorToRename> {
        Ok(self.value + arg)
    }
}

#[uniffi::export]
pub trait BindingTraitToRename: Send + Sync {
    fn trait_method(&self, value: i32) -> i32;
}

struct BindingTraitImpl {
    multiplier: i32,
}

impl BindingTraitToRename for BindingTraitImpl {
    fn trait_method(&self, value: i32) -> i32 {
        value * self.multiplier
    }
}

#[uniffi::export]
pub fn create_binding_trait_to_rename_impl(
    multiplier: i32,
) -> std::sync::Arc<dyn BindingTraitToRename> {
    std::sync::Arc::new(BindingTraitImpl { multiplier })
}
