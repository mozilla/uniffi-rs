pub struct UniffiOneType {
    pub sval: String,
}

pub enum UniffiOneEnum {
    One,
    Two,
}

include!(concat!(env!("OUT_DIR"), "/uniffi-one.uniffi.rs"));
