/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta::EnumShape;

use super::{Literal, Type};

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub module_path: String,
    pub remote: bool,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub self_type: Type,
}

// Represents an individual field on a Record.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub default: Option<Literal>,
    pub docstring: Option<String>,
}

/// Represents an enum with named variants, each of which may have named
/// and typed fields.
///
/// Enums are passed across the FFI by serializing to a bytebuffer, with a
/// i32 indicating the variant followed by the serialization of each field.
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub module_path: String,
    pub remote: bool,
    pub discr_type: Option<Type>,
    pub variants: Vec<Variant>,
    pub shape: EnumShape,
    pub non_exhaustive: bool,
    pub docstring: Option<String>,
    pub self_type: Type,
}

/// Represents an individual variant in an Enum.
///
/// Each variant has a name and zero or more fields.
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub discr: Literal,
    pub fields: Vec<Field>,
    pub enum_shape: EnumShape,
    pub docstring: Option<String>,
}
