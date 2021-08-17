pub struct UniffiOneType {
    pub sval: String,
}

include!(concat!(env!("OUT_DIR"), "/uniffi-one.uniffi.rs"));
