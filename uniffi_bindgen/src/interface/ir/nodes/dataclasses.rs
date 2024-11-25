/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone, PartialEq, Eq, Checksum)]
pub struct Record {
    pub(super) name: String,
    pub(super) module_path: String,
    pub(super) remote: bool,
    pub(super) fields: Vec<Field>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

// Represents an individual field on a Record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Checksum)]
pub struct Field {
    pub(super) name: String,
    pub(super) type_: Type,
    pub(super) default: Option<Literal>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

/// Represents an enum with named variants, each of which may have named
/// and typed fields.
///
/// Enums are passed across the FFI by serializing to a bytebuffer, with a
/// i32 indicating the variant followed by the serialization of each field.
#[derive(Debug, Clone, PartialEq, Eq, Checksum)]
pub struct Enum {
    pub(super) name: String,
    pub(super) module_path: String,
    pub(super) remote: bool,
    pub(super) discr_type: Option<Type>,
    pub(super) variants: Vec<Variant>,
    pub(super) shape: EnumShape,
    pub(super) non_exhaustive: bool,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

/// Represents an individual variant in an Enum.
///
/// Each variant has a name and zero or more fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Checksum)]
pub struct Variant {
    pub(super) name: String,
    pub(super) discr: Option<Literal>,
    pub(super) fields: Vec<Field>,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}
