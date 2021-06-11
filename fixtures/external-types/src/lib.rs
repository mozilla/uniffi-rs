// A trivial Guid implementation.
struct Guid(String);

// A simple `type JSONObject = serde_json::Value;` would work, except that
// we can't impl ViaFfi `impl doesn't use only types from inside the current crate`
struct JSONObject(serde_json::Value);

fn get_guid(guid: Option<Guid>) -> Guid {
    match guid {
        Some(guid) => guid,
        None => Guid("NewGuid".to_string()),
    }
}

fn get_string(s: Option<String>) -> String {
    match s {
        Some(s) => s,
        None => "NewString".to_string(),
    }
}

fn get_json_object(v: Option<JSONObject>) -> Option<JSONObject> {
    match v {
        Some(v) => Some(v),
        None => Some(JSONObject(serde_json::json!({"foo": "bar"}))),
    }
}

struct ExtTypes {
    guid: Guid,
    guids: Vec<Guid>,
    json: JSONObject,
    jsons: Vec<JSONObject>,
}

fn get_ext_types(vals: Option<ExtTypes>) -> ExtTypes {
    match vals {
        None => ExtTypes {
            guid: Guid("first-guid".to_string()),
            guids: vec![
                Guid("second-guid".to_string()),
                Guid("third-guid".to_string()),
            ],
            json: JSONObject(serde_json::json!({"foo": "bar"})),
            jsons: vec![
                JSONObject(serde_json::json!(["an", "array"])),
                JSONObject(serde_json::json!(3)),
            ],
        },
        Some(vals) => vals,
    }
}

// And we need a RustBufferViaFfi for them.
use anyhow::Result;

impl uniffi::RustBufferViaFfi for JSONObject {
    fn write(&self, buf: &mut Vec<u8>) {
        <String as uniffi::ViaFfi>::write(&self.0.to_string(), buf);
    }
    fn try_read(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self(serde_json::from_str(&<String as uniffi::ViaFfi>::try_read(buf)?).unwrap()))
    }
}

impl uniffi::RustBufferViaFfi for Guid {
    fn write(&self, buf: &mut Vec<u8>) {
        <String as uniffi::ViaFfi>::write(&self.0, buf);
    }
    fn try_read(buf: &mut &[u8]) -> Result<Self> {
        Ok(Self(<String as uniffi::ViaFfi>::try_read(buf)?))
    }
}

include!(concat!(env!("OUT_DIR"), "/external-types.uniffi.rs"));
