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

mod mod1 {
    #[derive(uniffi::Record)]
    struct Mod1Record { }

    #[uniffi::export]
    pub trait TraitInterface { }

    #[uniffi::export(with_foreign)]
    pub trait TraitInterfaceWithForeign { }

    #[uniffi::export(callback_interface)]
    pub trait CallbackInterface { }
}

mod glob_import_module {
    // This imports `u32`, but we don't directly know about that since we don't know the contents
    // of `std::primitive`
    use std::primitive::*;
}

// What happens when we try to use that from this crate?
use glob_import_module::u32;
