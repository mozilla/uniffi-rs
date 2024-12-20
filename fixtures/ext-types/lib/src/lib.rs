use custom_types::Handle;
use ext_types_custom::{ANestedGuid, Guid, Ouid};
use ext_types_external_crate::{
    ExternalCrateDictionary, ExternalCrateInterface, ExternalCrateNonExhaustiveEnum,
};
use std::sync::Arc;
use uniffi_one::{
    UniffiOneEnum, UniffiOneInterface, UniffiOneProcMacroType, UniffiOneTrait, UniffiOneType,
    UniffiOneUDLTrait,
};
use uniffi_sublib::SubLibType;
use url::Url;

// Remote types require a macro call in the Rust source
uniffi::use_remote_type!(custom_types::Url);

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

    pub ecd: ExternalCrateDictionary,
    pub ecnee: ExternalCrateNonExhaustiveEnum,
}

fn get_combined_type(existing: Option<CombinedType>) -> CombinedType {
    existing.unwrap_or_else(|| CombinedType {
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

        ecd: ExternalCrateDictionary { sval: "ecd".into() },
        ecnee: ExternalCrateNonExhaustiveEnum::One,
    })
}

// Not part of CombinedType as (a) object refs prevent equality testing and
// (b) it's not currently possible to refer to external traits in UDL.
#[derive(Default, uniffi::Record)]
pub struct ObjectsType {
    pub maybe_trait: Option<Arc<dyn UniffiOneTrait>>,
    pub maybe_interface: Option<Arc<UniffiOneInterface>>,
    pub sub: SubLibType,
}

#[uniffi::export]
fn get_objects_type(value: Option<ObjectsType>) -> ObjectsType {
    value.unwrap_or_default()
}

// A Custom type
fn get_url(url: Url) -> Url {
    url
}

fn get_urls(urls: Vec<Url>) -> Vec<Url> {
    urls
}

fn get_maybe_url(url: Option<Url>) -> Option<Url> {
    url
}

fn get_maybe_urls(urls: Vec<Option<Url>>) -> Vec<Option<Url>> {
    urls
}

#[uniffi::export]
fn get_imported_guid(guid: Guid) -> Guid {
    guid
}

#[uniffi::export]
fn get_imported_ouid(ouid: Ouid) -> Ouid {
    ouid
}

// external custom types wrapping external custom types.
#[uniffi::export]
fn get_imported_nested_guid(guid: Option<ANestedGuid>) -> ANestedGuid {
    guid.unwrap_or_else(|| ANestedGuid(Guid("nested".to_string())))
}

#[uniffi::export]
fn get_imported_nested_ouid(guid: Option<ANestedGuid>) -> ANestedGuid {
    guid.unwrap_or_else(|| ANestedGuid(Guid("nested".to_string())))
}

pub struct NestedExternalGuid(pub Guid);

#[uniffi::export]
fn get_nested_external_guid(nguid: Option<NestedExternalGuid>) -> NestedExternalGuid {
    nguid.unwrap_or_else(|| NestedExternalGuid(Guid("nested-external".to_string())))
}
uniffi::custom_newtype!(NestedExternalGuid, Guid);

// A local custom type wrapping an external imported procmacro type
pub struct NestedExternalOuid(pub Ouid);
uniffi::custom_newtype!(NestedExternalOuid, Ouid);

#[uniffi::export]
fn get_nested_external_ouid(ouid: Option<NestedExternalOuid>) -> NestedExternalOuid {
    ouid.unwrap_or_else(|| NestedExternalOuid(Ouid("nested-external-ouid".to_string())))
}

// A struct
fn get_uniffi_one_type(t: UniffiOneType) -> UniffiOneType {
    t
}

fn get_uniffi_one_types(ts: Vec<UniffiOneType>) -> Vec<UniffiOneType> {
    ts
}

fn get_maybe_uniffi_one_type(t: Option<UniffiOneType>) -> Option<UniffiOneType> {
    t
}

fn get_maybe_uniffi_one_types(ts: Vec<Option<UniffiOneType>>) -> Vec<Option<UniffiOneType>> {
    ts
}

// An enum
fn get_uniffi_one_enum(e: UniffiOneEnum) -> UniffiOneEnum {
    e
}

fn get_uniffi_one_enums(es: Vec<UniffiOneEnum>) -> Vec<UniffiOneEnum> {
    es
}

fn get_maybe_uniffi_one_enum(e: Option<UniffiOneEnum>) -> Option<UniffiOneEnum> {
    e
}

fn get_maybe_uniffi_one_enums(es: Vec<Option<UniffiOneEnum>>) -> Vec<Option<UniffiOneEnum>> {
    es
}

fn get_uniffi_one_interface() -> Arc<UniffiOneInterface> {
    Arc::new(UniffiOneInterface::new())
}

#[uniffi::export]
fn get_uniffi_one_trait(t: Option<Arc<dyn UniffiOneTrait>>) -> Option<Arc<dyn UniffiOneTrait>> {
    t
}

fn get_uniffi_one_proc_macro_type(t: UniffiOneProcMacroType) -> UniffiOneProcMacroType {
    t
}

fn get_external_crate_interface(val: String) -> Arc<ExternalCrateInterface> {
    Arc::new(ExternalCrateInterface::new(val))
}

fn get_uniffi_one_udl_trait(
    t: Option<Arc<dyn UniffiOneUDLTrait>>,
) -> Option<Arc<dyn UniffiOneUDLTrait>> {
    t
}

uniffi::include_scaffolding!("ext-types-lib");
