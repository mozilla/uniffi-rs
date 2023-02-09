use std::sync::atomic::{AtomicI32, Ordering};

pub struct UniffiOneType {
    pub sval: String,
}

pub enum UniffiOneEnum {
    One,
    Two,
}

#[derive(Default)]
pub struct UniffiOneInterface {
    current: AtomicI32,
}

impl UniffiOneInterface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self) -> i32 {
        self.current.fetch_add(1, Ordering::Relaxed) + 1
    }
}

pub trait UniffiOneCallbackInterface: Send + Sync {
    fn on_done(&self);
}

include!(concat!(env!("OUT_DIR"), "/uniffi-one.uniffi.rs"));
