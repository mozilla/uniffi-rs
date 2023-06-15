use std::sync::atomic::{AtomicI32, Ordering};

pub struct UniffiOneType {
    pub sval: String,
}

pub enum UniffiOneEnum {
    One,
    Two,
}

#[derive(uniffi::Record)]
pub struct UniffiOneProcMacroType {
    pub sval: String,
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

uniffi::include_scaffolding!("uniffi-one");
