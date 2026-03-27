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

uniffi::custom_type!(
    /// Custom type docstring
    JsonObject, String,
    {
        into: |obj| obj.serialize(),
        try_from: |s| s.deserialize(),
    }
);

uniffi::custom_type!(CustomRecord, TestRecord, {
    into: |r| r.into(),
    try_from: |r| r.try_into(),
});

uniffi::custom_newtype!(
    /// Custom newtype docstring
    Guid, u64
);

mod mod1 {
    use super::renamed_custom_type;

    #[derive(uniffi::Record)]
    struct Mod1Record { }

    #[uniffi::export]
    pub trait TraitInterface { }

    #[uniffi::export(with_foreign)]
    pub trait TraitInterfaceWithForeign { }

    #[uniffi::export(callback_interface)]
    pub trait CallbackInterface { }

    renamed_custom_type!(Handle, u64, {
        remote,
        into: |handle| handle.0,
        try_from: |v| Ok(Handle(v)),
    });
}

mod glob_import_module {
    // This imports `u32`, but we don't directly know about that since we don't know the contents
    // of `std::primitive`
    use std::primitive::*;
}

// What happens when we try to use that from this crate?
use glob_import_module::u32;
