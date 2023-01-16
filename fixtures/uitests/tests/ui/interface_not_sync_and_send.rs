
use std::cell::Cell;

// Unfortunately, path is relative to a temporary build directory :-/
uniffi_macros::generate_and_include_scaffolding!("../../../../fixtures/uitests/src/counter.udl");

fn main() { /* empty main required by `trybuild` */}

pub struct Counter {
    // This will fail to compile, because `Cell` is not `Sync`.
    value: Cell<u32>,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: Cell::new(0) }
    }

    pub fn increment(&self) -> u32 {
        let new_value = self.value.get() + 1;
        self.value.set(new_value);
        new_value
    }
}
