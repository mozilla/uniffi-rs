pub struct CrateTwoType {
    pub ival: i32,
}

include!(concat!(env!("OUT_DIR"), "/crate-two.uniffi.rs"));
