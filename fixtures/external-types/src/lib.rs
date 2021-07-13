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

// And we need a ViaFfi for them. Both of these boil down to "to and from a
// string" - this is the best markh could come up with, Maybe it should be
// a proc macro? Whatever, this serves the purpose for now.
use anyhow::Result;
use bytes::buf::{Buf, BufMut};
use std::convert::TryFrom;
use uniffi::{check_remaining, RustBuffer, ViaFfi};

#[macro_export]
macro_rules! viaffi_simple_string {
    ( $t:ident, $sel:ident, $to_string:expr, $from_string:expr ) => {
        unsafe impl ViaFfi for $t {
            type FfiType = RustBuffer;
            fn lower($sel) -> <$t as ViaFfi>::FfiType {
                RustBuffer::from_vec($to_string.into_bytes())
            }
            fn try_lift(v: Self::FfiType) -> Result<Self> {
                let v = v.destroy_into_vec();
                let s = unsafe { String::from_utf8_unchecked(v) };
                Ok($from_string(s)) // XXX - wire up errors
            }
            fn write<B: BufMut>(&$sel, buf: &mut B) {
                let s = &$to_string;
                let len = i32::try_from(s.len()).unwrap();
                buf.put_i32(len); // We limit strings to u32::MAX bytes
                buf.put(s.as_bytes());
            }
            fn try_read<B: Buf>(buf: &mut B) -> Result<Self> {
                check_remaining(buf, 4)?;
                let len = usize::try_from(buf.get_i32())?;
                check_remaining(buf, len)?;
                let bytes = &buf.chunk()[..len];
                let s = String::from_utf8(bytes.to_vec())?;
                buf.advance(len);
                Ok($from_string(s)) // XXX - wire up errors
            }
        }
    }
}

viaffi_simple_string!(JSONObject, self, self.0.to_string(), |s: String| Self(
    serde_json::from_str(&s).unwrap()
));
viaffi_simple_string!(Guid, self, self.0, |s| Self(s));

include!(concat!(env!("OUT_DIR"), "/external-types.uniffi.rs"));
