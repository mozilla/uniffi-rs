// A trivial guid.
pub struct Guid(pub String);

// This error is represented in the UDL.
#[derive(Debug, thiserror::Error)]
pub enum GuidError {
    #[error("The Guid is too short")]
    TooShort,
}

fn get_guid(guid: Option<Guid>) -> Guid {
    // This function doesn't return a Result, so all conversion errors are panics
    match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our TryFrom implementation already checked!"
            );
            guid
        }
        None => Guid("NewGuid".to_string()),
    }
}

fn try_get_guid(guid: Option<Guid>) -> std::result::Result<Guid, GuidError> {
    // This function itself always returns Ok - but it's declared as a Result
    // because the TryFrom implementation might return the Err as part of
    // turning the string into the Guid.
    Ok(match guid {
        Some(guid) => {
            assert!(
                !guid.0.is_empty(),
                "our TryFrom implementation failed to check for an empty GUID"
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

pub trait GuidCallback {
    fn run(&self, arg: Guid) -> Guid;
}

pub fn run_callback(callback: Box<dyn GuidCallback>) -> Guid {
    callback.run(Guid("callback-test-payload".into()))
}

impl TryFrom<String> for Guid {
    type Error = GuidError;

    // This is a "fixture" rather than an "example", so we are free to do things that don't really
    // make sense for real apps.
    fn try_from(val: String) -> Result<Self, Self::Error> {
        if val.is_empty() {
            Err(GuidError::TooShort)
        } else if val == "panic" {
            panic!("guid value caused a panic!");
        } else {
            Ok(Guid(val))
        }
    }
}

impl From<Guid> for String {
    fn from(guid: Guid) -> Self {
        guid.0
    }
}

include!(concat!(env!("OUT_DIR"), "/guid.uniffi.rs"));
