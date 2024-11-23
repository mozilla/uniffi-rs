/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta::EnumShape;

use super::{LanguageData, Literal, Type};

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
    pub lang_data: LanguageData,
}

// Represents an individual field on a Record.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub default: Option<Literal>,
    pub docstring: Option<String>,
    pub lang_data: LanguageData,
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
    pub lang_data: LanguageData,
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
    pub lang_data: LanguageData,
}

impl Record {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }

    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }
}

impl Enum {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }

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
