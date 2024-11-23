/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use toml::Value;

mod dataclasses;
mod ffi;
mod functions;
mod interfaces;
mod literal;
mod traits;
mod types;

pub use dataclasses::*;
pub use ffi::*;
pub use functions::*;
pub use interfaces::*;
pub use literal::*;
pub use traits::*;
pub use types::*;

/// Definition for custom types

/// This store exactly the same data as Type::Custom, since historically we've stuffed all data in
/// that variant.  Eventually we should consider removing some fields from Type::Custom to make it
/// work more like the other types
#[derive(Debug, Clone)]
pub struct CustomType {
    pub module_path: String,
    pub name: String,
    pub builtin: Type,
    pub self_type: Type,
    pub lang_data: LanguageData,
}

/// Definition for external types
///
/// This store exactly the same data as Type::External, since historically we've stuffed all data in
/// that variant.  Eventually we should consider removing some fields from Type::External to make it
/// work more like the other types
#[derive(Debug, Clone)]
pub struct ExternalType {
    pub module_path: String,
    pub name: String,
    pub namespace: String,
    pub kind: ExternalKind,
    pub self_type: Type,
    pub lang_data: LanguageData,
}

/// Container for language-specific data
///
/// We store one of these in the BindingsIr nodes.  It's initially empty, then bindings generators
/// store language-specific data as part of the specialization phase.
//
// LanguageData wraps a HashMap that stores TOML values.  Allows language bindings to store
// essentially whatever data they want. There are a couple alternatives to this, but they all have
// disadvantages:
//
// Creating new types for each bindings language (PyType, PyRecord, PyArgument, etc) would result
// in a lot of boilerplate since the language-specific types would mostly mirror the general ones.
// Also, you'd need to re-implement all the methods, like `throws()` and redefine trait
// implementations like `AsRef`.
//
// Making BindingsIr generic on the types would avoid that issue, but then you need to
// introduce a generic parameter for each node type that can be specialized. It quickly becomes
// very difficult to read the code.  It also gets very meta.  The docs would need to say things
// like "`T` is the generic param for the type that represents the type in the bindings
// language".
#[derive(Clone, Debug, Default)]
pub struct LanguageData(HashMap<String, Value>);

impl CustomType {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }
}

impl ExternalType {
    pub fn is_used_as_error(&self) -> bool {
        self.self_type.is_used_as_error
    }
}

impl LanguageData {
    /// Insert a value, serialization errors will result in panics
    pub fn insert<T: serde::Serialize>(&mut self, key: impl Into<String>, value: T) {
        let value = Value::try_from(value).expect("Error serializing value");
        self.0.insert(key.into(), value);
    }

    /// Try to get a value a value, deserialization errors will result in a panic
    pub fn get<'de, T: serde::Deserialize<'de>>(&self, key: impl AsRef<str>) -> Option<T> {
        self.0
            .get(key.as_ref())
            .map(|v| v.clone().try_into().expect("Error deserializing value"))
    }
}
