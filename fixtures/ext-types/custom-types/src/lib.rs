use std::collections::HashMap;

// A trivial guid, declared as `[Custom]` in the UDL.
pub struct Guid(pub String);

// Ditto, using a proc-macro.
pub struct Ouid(pub String);
uniffi::custom_newtype!(Ouid, String);

// This error is represented in the UDL.
#[derive(Debug, thiserror::Error)]
pub enum GuidError {
    #[error("The Guid is too short")]
    TooShort,
}

// This error is not represented in the UDL - it's only to be used internally (although
// for test purposes, we do allow this to leak out below.)
#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("Something unexpected went wrong")]
    Unexpected,
}

pub fn get_guid(guid: Option<Guid>) -> Guid {
    // This function doesn't return a Result, so all conversion errors are panics
    match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our custom type converter already checked!"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    }
}

fn try_get_guid(guid: Option<Guid>) -> std::result::Result<Guid, GuidError> {
    // This function itself always returns Ok - but it's declared as a Result
    // because the custom type converter might return the Err as part of
    // turning the string into the Guid.
    Ok(match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our custom type converter failed to check for an empty GUID"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    })
}

#[uniffi::export]
pub fn get_ouid(ouid: Option<Ouid>) -> Ouid {
    ouid.unwrap_or_else(|| Ouid("Ouid".to_string()))
}

pub struct GuidHelper {
    pub guid: Guid,
    pub guids: Vec<Guid>,
    pub maybe_guid: Option<Guid>,
}

fn get_guid_helper(vals: Option<GuidHelper>) -> GuidHelper {
    match vals {
        None => GuidHelper {
            guid: Guid("first-guid".to_string()),
            guids: vec![
                Guid("second-guid".to_string()),
                Guid("third-guid".to_string()),
            ],
            maybe_guid: None,
        },
        Some(vals) => vals,
    }
}

pub trait GuidCallback {
    fn run(&self, arg: Guid) -> Guid;
}

pub fn run_callback(callback: Box<dyn GuidCallback>) -> Guid {
    callback.run(Guid("callback-test-payload".into()))
}

uniffi::custom_type!(Guid, String, {
    try_lift: |val| {
        // This is a "fixture" rather than an "example", so we are free to do things that don't really
        // make sense for real apps.
        if val.is_empty() {
            Err(GuidError::TooShort.into())
        } else if val == "unexpected" {
            Err(InternalError::Unexpected.into())
        } else if val == "panic" {
            panic!("guid value caused a panic!");
        } else {
            Ok(Guid(val))
        }
    },
    lower: |obj| obj.0,
});

pub struct ANestedGuid(pub Guid);

uniffi::custom_newtype!(ANestedGuid, Guid);

#[uniffi::export]
fn get_nested_guid(nguid: Option<ANestedGuid>) -> ANestedGuid {
    nguid.unwrap_or_else(|| ANestedGuid(Guid("ANestedGuid".to_string())))
}

// Dependent types might come in the "wrong" order - #2067 triggered
// a bug here because of the alphabetical order of these types.
pub struct ANestedOuid(pub Ouid);
uniffi::custom_newtype!(ANestedOuid, Ouid);

#[uniffi::export]
fn get_nested_ouid(nouid: Option<ANestedOuid>) -> ANestedOuid {
    nouid.unwrap_or_else(|| ANestedOuid(Ouid("ANestedOuid".to_string())))
}

// Dependent types that are nested deeper inside of other types
// need to also be taken into account when ordering aliases.
#[derive(PartialEq, Eq, Hash)]
pub struct StringWrapper(pub String);
uniffi::custom_newtype!(StringWrapper, String);

pub struct IntWrapper(pub u32);
uniffi::custom_newtype!(IntWrapper, u32);

pub struct MapUsingStringWrapper(pub HashMap<StringWrapper, IntWrapper>);
uniffi::custom_newtype!(MapUsingStringWrapper, HashMap<StringWrapper, IntWrapper>);

#[uniffi::export]
fn get_map_using_string_wrapper(maybe_map: Option<MapUsingStringWrapper>) -> MapUsingStringWrapper {
    maybe_map.unwrap_or_else(|| MapUsingStringWrapper(HashMap::new()))
}

// And custom types around other objects.
#[derive(uniffi::Object)]
pub struct InnerObject;

#[uniffi::export]
impl InnerObject {
    #[uniffi::constructor]
    fn new() -> Self {
        Self
    }
}

pub struct NestedObject(pub std::sync::Arc<InnerObject>);
uniffi::custom_newtype!(NestedObject, std::sync::Arc<InnerObject>);

#[uniffi::export]
pub fn get_nested_object(n: NestedObject) -> NestedObject {
    n
}

#[derive(uniffi::Record)]
pub struct InnerRecord {
    i: i32,
}

pub struct NestedRecord(pub InnerRecord);
uniffi::custom_newtype!(NestedRecord, InnerRecord);

#[uniffi::export]
pub fn get_nested_record(n: NestedRecord) -> NestedRecord {
    n
}

uniffi::include_scaffolding!("custom_types");
