// We just expose types here - our fixtures 'lib' crate will reference them
// and exposes functions to work with them.

pub struct UniffiOneType {
    pub sval: String,
}

pub enum Animal {
    Dog,
    Cat,
}

pub enum IpAddr {
    V4 {q1: u8, q2: u8, q3: u8, q4: u8},
    V6 {addr: String},
}

include!(concat!(env!("OUT_DIR"), "/uniffi-one.uniffi.rs"));
