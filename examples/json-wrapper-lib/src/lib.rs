use json_wrapper_ext::JsonObject;

pub fn get(mut obj: JsonObject, key: String) -> Option<JsonObject> {
    obj.get_mut(key).map(|v| v.take())
}

include!(concat!(env!("OUT_DIR"), "/json-wrapper-lib.uniffi.rs"));
