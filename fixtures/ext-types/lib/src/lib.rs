use custom_types::Handle;
use ext_types_guid::Guid;
use std::sync::Arc;
use uniffi_one::{UniffiOneCallbackInterface, UniffiOneEnum, UniffiOneInterface, UniffiOneType};
use url::Url;

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
    })
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

fn use_uniffi_one_callback_interface(iface: Box<dyn UniffiOneCallbackInterface>) -> String {
    return iface.on_done("fromrust".to_string());
}

include!(concat!(env!("OUT_DIR"), "/ext-types-lib.uniffi.rs"));
