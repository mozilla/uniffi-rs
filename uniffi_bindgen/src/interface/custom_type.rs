/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta::Checksum;

use super::Type;

/// Represents a custom type definition, analogous to [`super::Record`], [`super::Enum`], etc.
///
/// It's the "type definition" for a `Type::Custom { .. }`. It adds things like a docstring.
#[derive(Debug, Clone, Checksum)]
pub struct CustomType {
    pub name: String,
    pub module_path: String,
    pub builtin: Type,
    #[checksum_ignore]
    pub(super) docstring: Option<String>,
}

impl CustomType {
    pub fn docstring(&self) -> Option<&str> {
        self.docstring.as_deref()
    }
}

impl From<uniffi_meta::CustomTypeMetadata> for CustomType {
    fn from(meta: uniffi_meta::CustomTypeMetadata) -> Self {
        Self {
            name: meta.name,
            module_path: meta.module_path,
            builtin: meta.builtin,
            docstring: meta.docstring,
        }
    }
}
