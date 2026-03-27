use uniffi::Record;

// Record with the same name as a macro is okay, since macros live in the macro namespace and types
// live in the type namespace
#[derive(Record)]
pub struct Record { }

// Conflict between two imported names
mod mod1 {
    #[derive(Record)]
    pub struct RenamedRecordConflict { }

    #[derive(Record)]
    pub struct RenamedRecordConflict2 { }
}

use mod1::{RenamedRecordConflict, RenamedRecordConflict2 as RenamedRecordConflict};

// when there's a conflict between a glob import and regular item, the regular item wins
mod mod2 {
    #[derive(Record)]
    pub struct ItemGlobConflict { }
}

use mod2::*;

#[derive(Record)]
pub struct ItemGlobConflict { }

// 2 glob imports with the same name should be an error
mod mod3 {
    #[derive(Record)]
    pub struct GlobGlobConflict { }

    mod mod4 {
        #[derive(Record)]
        pub struct GlobGlobConflict { }
    }
}

use mod3::*;
use mod3::mod4::*;

// Complete corner case: record that shares the same name with the custom_type! macro.
//
// This is not an error in Rust, since macros live in a different namespace.  It doesn't need to
// be a UniFFI error either since we don't export the custom_type! macro to the foreign language.
// However, the current code does consider this a name conflict and it doesn't seem worth it to
// fix.
use uniffi::custom_type as CustomTypeConflict;

#[derive(Record)]
pub struct CustomTypeConflict { }
