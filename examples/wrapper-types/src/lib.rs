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

fn objectify(name: String, obj: serde_json::Value) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(name, obj);
    serde_json::Value::Object(map)
}

// And we also have a trivial "Handle" type which wraps an i64.
pub struct Handle(pub i64);

fn get_next_handle(handle: Handle) -> Handle {
    Handle(handle.0 + 1)
}

impl UniffiCustomTypeWrapper for Handle {
    type Wrapped = i64;

    fn wrap(val: Self::Wrapped) -> uniffi::Result<Self> {
        Ok(Handle(val))
    }

    fn unwrap(obj: Self) -> Self::Wrapped {
        obj.0
    }
}

// And a little struct and function that ties them together.
pub struct WrappedTypesDemo {
    json: serde_json::Value,
    handle: Handle,
}

pub fn get_wrapped_types_demo(v: Option<WrappedTypesDemo>) -> WrappedTypesDemo {
    v.unwrap_or_else(|| WrappedTypesDemo {
        json: serde_json::json!({"demo": "string"}),
        handle: Handle(123),
    })
}

include!(concat!(env!("OUT_DIR"), "/wrapper-types.uniffi.rs"));
