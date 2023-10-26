use custom_types::Handle;
use ext_types_guid::Guid;
use std::sync::Arc;
use uniffi_one::{UniffiOneEnum, UniffiOneInterface, UniffiOneProcMacroType, UniffiOneType};
use url::Url;

uniffi::use_udl_record!(uniffi_one, UniffiOneType);
uniffi::use_udl_enum!(uniffi_one, UniffiOneEnum);
uniffi::use_udl_object!(uniffi_one, UniffiOneInterface);
uniffi::use_udl_record!(ext_types_guid, Guid);
uniffi::use_udl_record!(custom_types, Url);
uniffi::use_udl_record!(custom_types, Handle);

#[derive(uniffi::Record)]
pub struct CombinedType {
    pub uoe: UniffiOneEnum,
    pub uot: UniffiOneType,
    pub uots: Vec<UniffiOneType>,
    pub maybe_uot: Option<UniffiOneType>,

    pub guid: Guid,
    pub guids: Vec<Guid>,
    pub maybe_guid: Option<Guid>,

    pub url: Url,
    pub urls: Vec<Url>,
    pub maybe_url: Option<Url>,

    pub handle: Handle,
    pub handles: Vec<Handle>,
    pub maybe_handle: Option<Handle>,
}

#[uniffi::export]
fn get_combined_type(value: Option<CombinedType>) -> CombinedType {
    value.unwrap_or_else(|| CombinedType {
        uoe: UniffiOneEnum::One,
        uot: UniffiOneType {
            sval: "hello".to_string(),
        },
        uots: vec![
            UniffiOneType {
                sval: "first of many".to_string(),
            },
            UniffiOneType {
                sval: "second of many".to_string(),
            },
        ],
        maybe_uot: None,

        guid: Guid("a-guid".into()),
        guids: vec![Guid("b-guid".into()), Guid("c-guid".into())],
        maybe_guid: None,

        url: Url::parse("http://example.com/").unwrap(),
        urls: vec![],
        maybe_url: None,

        handle: Handle(123),
        handles: vec![Handle(1), Handle(2), Handle(3)],
        maybe_handle: Some(Handle(4)),
    })
}

// A Custom type
#[uniffi::export]
fn get_url(url: Url) -> Url {
    url
}

#[uniffi::export]
fn get_urls(urls: Vec<Url>) -> Vec<Url> {
    urls
}

#[uniffi::export]
fn get_maybe_url(url: Option<Url>) -> Option<Url> {
    url
}

#[uniffi::export]
fn get_maybe_urls(urls: Vec<Option<Url>>) -> Vec<Option<Url>> {
    urls
}

// A struct
#[uniffi::export]
fn get_uniffi_one_type(t: UniffiOneType) -> UniffiOneType {
    t
}

// Test using a type defined in a proc-macro in another crate
#[uniffi::export]
fn get_uniffi_one_proc_macro_type(t: UniffiOneProcMacroType) -> UniffiOneProcMacroType {
    t
}

#[uniffi::export]
fn get_uniffi_one_types(ts: Vec<UniffiOneType>) -> Vec<UniffiOneType> {
    ts
}

#[uniffi::export]
fn get_maybe_uniffi_one_type(t: Option<UniffiOneType>) -> Option<UniffiOneType> {
    t
}

#[uniffi::export]
fn get_maybe_uniffi_one_types(ts: Vec<Option<UniffiOneType>>) -> Vec<Option<UniffiOneType>> {
    ts
}

// An enum
#[uniffi::export]
fn get_uniffi_one_enum(e: UniffiOneEnum) -> UniffiOneEnum {
    e
}

#[uniffi::export]
fn get_uniffi_one_enums(es: Vec<UniffiOneEnum>) -> Vec<UniffiOneEnum> {
    es
}

#[uniffi::export]
fn get_maybe_uniffi_one_enum(e: Option<UniffiOneEnum>) -> Option<UniffiOneEnum> {
    e
}

#[uniffi::export]
fn get_maybe_uniffi_one_enums(es: Vec<Option<UniffiOneEnum>>) -> Vec<Option<UniffiOneEnum>> {
    es
}

#[uniffi::export]
fn get_uniffi_one_interface() -> Arc<UniffiOneInterface> {
    Arc::new(UniffiOneInterface::new())
}

// Some custom types via macros.
// Another guid - here we use a regular struct.
pub struct Uuid {
    val: String,
}

// Tell UniFfi we want to use am UniffiCustomTypeConverter to go to and
// from a String.
//  Note this could be done even if the above `struct` defn was external.
uniffi::custom_type!(Uuid, String);

impl UniffiCustomTypeConverter for Uuid {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Uuid { val })
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.val
    }
}

mod submodule {
    // A custom type using the "newtype" idiom.
    // Uniffi can generate the UniffiCustomTypeConverter for us.
    pub struct NewtypeHandle(pub(super) i64);

    // Uniffi can generate the UniffiCustomTypeConverter for us too.
    uniffi::custom_newtype!(NewtypeHandle, i64);
}
pub use submodule::NewtypeHandle;

#[uniffi::export]
fn get_uuid(u: Option<Uuid>) -> Uuid {
    u.unwrap_or_else(|| Uuid {
        val: "new".to_string(),
    })
}

#[uniffi::export]
fn get_uuid_value(u: Uuid) -> String {
    u.val
}

#[uniffi::export]
fn get_newtype_handle(u: Option<NewtypeHandle>) -> NewtypeHandle {
    u.unwrap_or(NewtypeHandle(42))
}

#[uniffi::export]
fn get_newtype_handle_value(u: NewtypeHandle) -> i64 {
    u.0
}

#[uniffi::export]
fn get_guid_procmacro(g: Option<Guid>) -> Guid {
    ext_types_guid::get_guid(g)
}

uniffi::setup_scaffolding!("imported_types_lib");
