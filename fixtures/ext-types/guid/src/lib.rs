// A trivial guid.
pub struct Guid(pub String);

fn get_guid(guid: Option<Guid>) -> Guid {
    match guid {
        Some(guid) => guid,
        None => Guid("NewGuid".to_string()),
    }
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

    fn wrap(val: Self::Wrapped) -> uniffi::Result<Self> {
        Ok(Guid(val))
    }

    fn unwrap(obj: Self) -> Self::Wrapped {
        obj.0
    }
}

include!(concat!(env!("OUT_DIR"), "/guid.uniffi.rs"));
