use uniffi::FfiConverter;

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
}

fn get_guid_helper(vals: Option<GuidHelper>) -> GuidHelper {
    match vals {
        None => GuidHelper {
            guid: Guid("first-guid".to_string()),
            guids: vec![
                Guid("second-guid".to_string()),
                Guid("third-guid".to_string()),
            ],
        },
        Some(vals) => vals,
    }
}

// The name is hard-coded based on the 2 types involved.  Definitely open to a suggestions for how
// we generate the name.
struct FFIConverterWrappingGuidString;

impl FFIConverterWrappingGuidString {
    pub fn to_wrapper(guid: Guid) -> String {
        guid.0
    }

    pub fn from_wrapper(string: String) -> uniffi::Result<Guid> {
        Ok(Guid(string))
    }
}

// Imagine this being created by a derive macro
unsafe impl FfiConverter for FFIConverterWrappingGuidString {
    type RustType = Guid;
    type FfiType = uniffi::RustBuffer;

    fn lower(obj: Guid) -> uniffi::RustBuffer {
        <String as FfiConverter>::lower(Self::to_wrapper(obj))
    }

    fn try_lift(v: uniffi::RustBuffer) -> uniffi::Result<Guid> {
        Self::from_wrapper(<String as FfiConverter>::try_lift(v)?)
    }

    fn write(obj: Guid, buf: &mut Vec<u8>) {
        <String as FfiConverter>::write(Self::to_wrapper(obj), buf);
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::Result<Guid> {
        Self::from_wrapper(<String as FfiConverter>::try_read(buf)?)
    }
}


include!(concat!(env!("OUT_DIR"), "/guid.uniffi.rs"));
