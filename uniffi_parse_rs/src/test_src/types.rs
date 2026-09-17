use uniffi::custom_type as renamed_custom_type;
use std::primitive::u64 as RenamedU64;
use std::primitive;
use std::collections::HashMap;

#[derive(uniffi::Record)]
struct TestRecord { }

#[derive(uniffi::Enum)]
enum TestEnum { }

#[derive(uniffi::Error)]
enum TestError { }

#[derive(uniffi::Object)]
struct TestInterface { }

pub type JsonObject = serde_json::Value;

uniffi::custom_type!(
    /// Custom type docstring
    JsonObject, String,
    {
        remote,
        lower: |obj| obj.serialize(),
        try_lift: |s| s.deserialize(),
    }
);

pub struct CustomRecord;
pub struct CustomRecord2;
pub struct CustomRecord3;
pub struct CustomRecord4;

uniffi::custom_type!(CustomRecord, TestRecord, {
    lower: |r| r.into(),
    try_lift: |r| r.try_into(),
});

pub struct Guid;

uniffi::custom_newtype!(
    /// Custom newtype docstring
    Guid, u64
);

pub mod mod1 {
    use super::renamed_custom_type;

    #[derive(uniffi::Record)]
    pub struct Mod1Record { }

    #[uniffi::export]
    pub trait TraitInterface { }

    #[uniffi::export(foreign)]
    pub trait TraitInterfaceForeignOnly { }

    #[uniffi::export(rust, foreign)]
    pub trait TraitInterfaceWithForeign { }

    #[uniffi::export(callback_interface)]
    pub trait CallbackInterface { }

    renamed_custom_type!(Handle, u64, {
        remote,
        lower: |handle| handle.0,
        try_lift: |v| Ok(Handle(v)),
    });

    // Test the custom type macro being in a different module from the type itself.
    // Users should be able to the type imported from the original module.
    //
    // Note: this doesn't currently work for remote types
    // (https://github.com/mozilla/uniffi-rs/issues/2994)
    use super::CustomRecord2;
    uniffi::custom_type!(CustomRecord2, u64, {
        lower: |r| r.into(),
        try_lift: |r| r.try_into(),
    });

    // Test custom types where the source type and the bridge type are defined in different modules
    use super::CustomRecord3;
    #[derive(uniffi::Record)]
    pub struct CustomRecord3BridgeType {
        value: u64,
    }
    uniffi::custom_type!(CustomRecord3, CustomRecord3BridgeType, {
        lower: |r| r.into(),
        try_lift: |r| r.try_into(),
    });

    // Test custom types where the source type is an alias
    pub type CustomRecord4Alias = crate::CustomRecord4;
    uniffi::custom_type!(CustomRecord4Alias, u64, {
        lower: |r| r.into(),
        try_lift: |r| r.try_into(),
    });
}

pub mod mod2 {
    // One more custom type test:
    // Custom type imports from another module should still resolve to the custom type.
    pub use super::CustomRecord2;

    // Let's also test a two-level type alias to a custom type here.
    pub type CustomRecord4AliasAlias = super::mod1::CustomRecord4Alias;
}

mod glob_import_module {
    // This imports `u32`, but we don't directly know about that since we don't know the contents
    // of `std::primitive`
    use std::primitive::*;
}

// What happens when we try to use that from this crate?
use glob_import_module::u32;
