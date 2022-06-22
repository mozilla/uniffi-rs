/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// Name types
///
/// These all use the newtype pattern to wrap a string.  This allows renderers to customize the
/// naming style when rendering them.
use serde::{Deserialize, Serialize};
use std::ops::Deref;

macro_rules! define_names {
    ($($type_name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
            #[serde(tag = "ir_type")]
            pub struct $type_name {
                pub name: String,
            }

            impl $type_name {
                pub fn new(name: impl Into<String>) -> Self {
                    Self { name: name.into() }
                }

                pub fn equals(&self, other: &str) -> bool {
                    self.name == other
                }
            }

            impl Deref for $type_name {
                type Target = String;

                fn deref(&self) -> &String {
                    &self.name
                }
            }
            impl std::fmt::Display for $type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.name)
                }
            }

            impl From<String> for $type_name {
                fn from(name: String) -> Self {
                    Self { name }
                }
            }

            impl From<&str> for $type_name {
                fn from(name: &str) -> Self {
                    Self { name: name.to_string() }
                }
            }
        )*
    }
}

define_names!(
    ArgName,
    FunctionName,
    FieldName,
    ClassName,
    CStructName,
    VarName
);
