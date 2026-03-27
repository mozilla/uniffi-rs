// Import type from another module.
//
// The `self` import here is to try to cycle detection.
use paths::{self, TestRecord, mod1::mod2};

#[derive(uniffi::Record)]
struct AmbiguousRecord { }
