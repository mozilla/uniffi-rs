// A trivial guid.
pub struct Guid(pub String);

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

fn get_guid(guid: Option<Guid>) -> Guid {
    // This function doesn't return a Result, so all conversion errors are panics
    match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our UniffiCustomTypeWrapper already checked!"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    }
}

fn try_get_guid(guid: Option<Guid>) -> std::result::Result<Guid, GuidError> {
    // This function itself always returns Ok - but it's declared as a Result
    // because the UniffiCustomTypeWrapper might return the Err as part of
    // turning the string into the Guid.
    Ok(match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our UniffiCustomTypeWrapper already checked!"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    })
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

impl UniffiCustomTypeWrapper for Guid {
    type Wrapped = String;

    // This is a "fixture" rather than an "example", so we are free to do things that don't really
    // make sense for real apps.
    fn wrap(val: Self::Wrapped) -> uniffi::Result<Self> {
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

    fn unwrap(obj: Self) -> Self::Wrapped {
        obj.0
    }
}

include!(concat!(env!("OUT_DIR"), "/guid.uniffi.rs"));
