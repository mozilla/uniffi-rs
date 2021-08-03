pub struct CrateOneType {
    pub sval: String,
}

include!(concat!(env!("OUT_DIR"), "/crate-one.uniffi.rs"));
