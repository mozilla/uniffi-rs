/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_internal_macros::AsType;
use uniffi_meta::EnumShape;

use super::{AsType, Type};

/// Represents a "data class" style object, for passing around complex values.
///
/// In the FFI these are represented as a byte buffer, which one side explicitly
/// serializes the data into and the other serializes it out of. So I guess they're
/// kind of like "pass by clone" values.
#[derive(Debug, Clone, AsType)]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub docstring: Option<String>,
    pub self_type: Type,
}

// Represents an individual field on a Record.
#[derive(Debug, Clone, AsType)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub default: Option<String>,
    pub docstring: Option<String>,
}

/// Represents an enum with named variants, each of which may have named
/// and typed fields.
///
/// Enums are passed across the FFI by serializing to a bytebuffer, with a
/// i32 indicating the variant followed by the serialization of each field.
#[derive(Debug, Clone, AsType)]
pub struct Enum {
    pub name: String,
    pub discr_type: Option<Type>,
    pub variants: Vec<Variant>,
    pub shape: EnumShape,
    pub docstring: Option<String>,
    pub self_type: Type,
}

/// Represents an individual variant in an Enum.
///
/// Each variant has a name and zero or more fields.
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub discr: String,
    pub fields: Vec<Field>,
    pub enum_shape: EnumShape,
    pub docstring: Option<String>,
}

impl Record {
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }
}

impl Enum {
    pub fn is_flat(&self) -> bool {
        match self.shape {
            EnumShape::Error { flat } => flat,
            EnumShape::Enum => self.variants.iter().all(|v| v.fields.is_empty()),
        }
    }

    pub fn is_flat_error(&self) -> bool {
        self.shape.is_flat_error()
    }
}

impl Variant {
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    pub fn has_nameless_fields(&self) -> bool {
        self.fields.iter().any(|f| f.name.is_empty())
    }
}

impl Field {
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }
}
