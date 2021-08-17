// We use `JsonObject` in our UDL. It moves to and from Uniffi bindings via a string.
pub type JsonObject = serde_json::Value;

// We must implement the UniffiCustomTypeWrapper trait.
impl UniffiCustomTypeWrapper for JsonObject {
    type Wrapped = String;

    fn wrap(val: Self::Wrapped) -> uniffi::Result<Self> {
        Ok(serde_json::from_str(&val)?)
    }

    fn unwrap(obj: Self) -> Self::Wrapped {
        obj.to_string()
    }
}

include!(concat!(env!("OUT_DIR"), "/json-wrapper-ext.uniffi.rs"));
