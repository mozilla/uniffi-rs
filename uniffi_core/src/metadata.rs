/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Pack UniFFI interface metadata into byte arrays
//!
//! In order to generate foreign bindings, we store interface metadata inside the library file
//! using exported static byte arrays.  The foreign bindings code reads that metadata from the
//! library files and generates bindings based on that.
//!
//! The metadata static variables are generated by the proc-macros, which is an issue because the
//! proc-macros don't have knowledge of the entire interface -- they can only see the item they're
//! wrapping.  For example, when a proc-macro sees a type name, it doesn't know anything about the
//! actual type: it could be a Record, an Enum, or even a type alias for a `Vec<>`/`Result<>`.
//!
//! This module helps bridge the gap by providing tools that allow the proc-macros to generate code
//! to encode the interface metadata:
//!   - A set of const functions to build up metadata buffers with const expressions
//!   - The `export_static_metadata_var!` macro, which creates the static variable from a const metadata
//!     buffer.
//!   - The `FfiConverter::TYPE_ID_META` const which encodes an identifier for that type in a
//!     metadata buffer.
//!
//! `uniffi_bindgen::macro_metadata` contains the code to read the metadata from a library file.
//! `fixtures/metadata` has the tests.

/// Metadata constants, make sure to keep this in sync with copy in `uniffi_meta::reader`
pub mod codes {
    // Top-level metadata item codes
    pub const FUNC: u8 = 0;
    pub const METHOD: u8 = 1;
    pub const RECORD: u8 = 2;
    pub const ENUM: u8 = 3;
    pub const INTERFACE: u8 = 4;
    pub const ERROR: u8 = 5;
    pub const NAMESPACE: u8 = 6;
    pub const CONSTRUCTOR: u8 = 7;
    pub const UDL_FILE: u8 = 8;
    pub const CALLBACK_INTERFACE: u8 = 9;
    pub const TRAIT_METHOD: u8 = 10;
    pub const UNIFFI_TRAIT: u8 = 11;
    pub const UNKNOWN: u8 = 255;

    // Type codes
    pub const TYPE_U8: u8 = 0;
    pub const TYPE_U16: u8 = 1;
    pub const TYPE_U32: u8 = 2;
    pub const TYPE_U64: u8 = 3;
    pub const TYPE_I8: u8 = 4;
    pub const TYPE_I16: u8 = 5;
    pub const TYPE_I32: u8 = 6;
    pub const TYPE_I64: u8 = 7;
    pub const TYPE_F32: u8 = 8;
    pub const TYPE_F64: u8 = 9;
    pub const TYPE_BOOL: u8 = 10;
    pub const TYPE_STRING: u8 = 11;
    pub const TYPE_OPTION: u8 = 12;
    pub const TYPE_RECORD: u8 = 13;
    pub const TYPE_ENUM: u8 = 14;
    // 15 no longer used.
    pub const TYPE_INTERFACE: u8 = 16;
    pub const TYPE_VEC: u8 = 17;
    pub const TYPE_HASH_MAP: u8 = 18;
    pub const TYPE_SYSTEM_TIME: u8 = 19;
    pub const TYPE_DURATION: u8 = 20;
    pub const TYPE_CALLBACK_INTERFACE: u8 = 21;
    pub const TYPE_CUSTOM: u8 = 22;
    pub const TYPE_RESULT: u8 = 23;
    pub const TYPE_FUTURE: u8 = 24;
    pub const TYPE_FOREIGN_EXECUTOR: u8 = 25;
    pub const TYPE_UNIT: u8 = 255;

    // Literal codes for LiteralMetadata - note that we don't support
    // all variants in the "emit/reader" context.
    pub const LIT_STR: u8 = 0;
    pub const LIT_INT: u8 = 1;
    pub const LIT_FLOAT: u8 = 2;
    pub const LIT_BOOL: u8 = 3;
    pub const LIT_NULL: u8 = 4;
}

const BUF_SIZE: usize = 4096;

// This struct is a kludge around the fact that Rust const generic support doesn't quite handle our
// needs.
//
// We'd like to have code like this in `FfiConverter`:
//
// ```
//   const TYPE_ID_META_SIZE: usize;
//   const TYPE_ID_META: [u8, Self::TYPE_ID_META_SIZE];
// ```
//
// This would define a metadata buffer, correctly size for the data needed. However, associated
// consts as generic params aren't supported yet.
//
// To work around this, we use `const MetadataBuffer` values, which contain fixed-sized buffers
// with enough capacity to store our largest metadata arrays.  Since the `MetadataBuffer` values
// are const, they're only stored at compile time and the extra bytes don't end up contributing to
// the final binary size.  This was tested on Rust `1.66.0` with `--release` by increasing
// `BUF_SIZE` and checking the compiled library sizes.
#[derive(Debug)]
pub struct MetadataBuffer {
    pub bytes: [u8; BUF_SIZE],
    pub size: usize,
}

impl MetadataBuffer {
    pub const fn new() -> Self {
        Self {
            bytes: [0; BUF_SIZE],
            size: 0,
        }
    }

    pub const fn from_code(value: u8) -> Self {
        Self::new().concat_value(value)
    }

    // Concatenate another buffer to this one.
    //
    // This consumes self, which is convenient for the proc-macro code and also allows us to avoid
    // allocated an extra buffer.
    pub const fn concat(mut self, other: MetadataBuffer) -> MetadataBuffer {
        assert!(self.size + other.size <= BUF_SIZE);
        // It would be nice to use `copy_from_slice()`, but that's not allowed in const functions
        // as of Rust 1.66.
        let mut i = 0;
        while i < other.size {
            self.bytes[self.size] = other.bytes[i];
            self.size += 1;
            i += 1;
        }
        self
    }

    // Concatenate a `u8` value to this buffer
    //
    // This consumes self, which is convenient for the proc-macro code and also allows us to avoid
    // allocated an extra buffer.
    pub const fn concat_value(mut self, value: u8) -> Self {
        assert!(self.size < BUF_SIZE);
        self.bytes[self.size] = value;
        self.size += 1;
        self
    }

    // Concatenate a `u32` value to this buffer
    //
    // This consumes self, which is convenient for the proc-macro code and also allows us to avoid
    // allocated an extra buffer.
    pub const fn concat_u32(mut self, value: u32) -> Self {
        assert!(self.size + 4 <= BUF_SIZE);
        // store the value as little-endian
        self.bytes[self.size] = value as u8;
        self.bytes[self.size + 1] = (value >> 8) as u8;
        self.bytes[self.size + 2] = (value >> 16) as u8;
        self.bytes[self.size + 3] = (value >> 24) as u8;
        self.size += 4;
        self
    }

    // Concatenate a `bool` value to this buffer
    //
    // This consumes self, which is convenient for the proc-macro code and also allows us to avoid
    // allocated an extra buffer.
    pub const fn concat_bool(self, value: bool) -> Self {
        self.concat_value(value as u8)
    }

    // Concatenate a string to this buffer.
    //
    // Strings are encoded as a `u8` length, followed by the utf8 data.
    //
    // This consumes self, which is convenient for the proc-macro code and also allows us to avoid
    // allocated an extra buffer.
    pub const fn concat_str(mut self, string: &str) -> Self {
        assert!(string.len() < 256);
        assert!(self.size + string.len() < BUF_SIZE);
        self.bytes[self.size] = string.len() as u8;
        self.size += 1;
        let bytes = string.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            self.bytes[self.size] = bytes[i];
            self.size += 1;
            i += 1;
        }
        self
    }

    // Create an array from this MetadataBuffer
    //
    // SIZE should always be `self.size`.  This is part of the kludge to hold us over until Rust
    // gets better const generic support.
    pub const fn into_array<const SIZE: usize>(self) -> [u8; SIZE] {
        let mut result: [u8; SIZE] = [0; SIZE];
        let mut i = 0;
        while i < SIZE {
            result[i] = self.bytes[i];
            i += 1;
        }
        result
    }

    // Create a checksum from this MetadataBuffer
    //
    // This is used by the bindings code to verify that the library they link to is the same one
    // that the bindings were generated from.
    pub const fn checksum(&self) -> u16 {
        calc_checksum(&self.bytes, self.size)
    }
}

impl AsRef<[u8]> for MetadataBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.size]
    }
}

// Create a checksum for a MetadataBuffer
//
// This is used by the bindings code to verify that the library they link to is the same one
// that the bindings were generated from.
pub const fn checksum_metadata(buf: &[u8]) -> u16 {
    calc_checksum(buf, buf.len())
}

const fn calc_checksum(bytes: &[u8], size: usize) -> u16 {
    // Taken from the fnv_hash() function from the FNV crate (https://github.com/servo/rust-fnv/blob/master/lib.rs).
    // fnv_hash() hasn't been released in a version yet.
    const INITIAL_STATE: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = INITIAL_STATE;
    let mut i = 0;
    while i < size {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(PRIME);
        i += 1;
    }
    // Convert the 64-bit hash to a 16-bit hash by XORing everything together
    (hash ^ (hash >> 16) ^ (hash >> 32) ^ (hash >> 48)) as u16
}
