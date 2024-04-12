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
                "our UniffiCustomTypeConverter already checked!"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    }
}

fn try_get_guid(guid: Option<Guid>) -> std::result::Result<Guid, GuidError> {
    // This function itself always returns Ok - but it's declared as a Result
    // because the UniffiCustomTypeConverter might return the Err as part of
    // turning the string into the Guid.
    Ok(match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our UniffiCustomTypeConverter failed to check for an empty GUID"
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

impl UniffiCustomTypeConverter for Guid {
    type Builtin = String;

    // This is a "fixture" rather than an "example", so we are free to do things that don't really
    // make sense for real apps.
    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        if val.is_empty() {
            Err(GuidError::TooShort.into())
        } else if val == "unexpected" {
            Err(InternalError::Unexpected.into())
        } else if val == "panic" {
            panic!("guid value caused a panic!");
        } else {
            Ok(Guid(val))
        }
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}

uniffi::include_scaffolding!("custom_types");
