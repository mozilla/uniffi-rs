/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, Ident};
use uniffi_meta::CustomTypeMetadata;

use crate::{paths::LookupCache, Ir, RPath, Result};

pub struct CustomType {
    pub ident: Ident,
    pub builtin: syn::Type,
}

impl CustomType {
    pub fn custom_type_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<CustomTypeMetadata> {
        Ok(CustomTypeMetadata {
            module_path: module_path.path_string(),
            name: self.ident.unraw().to_string(),
            builtin: module_path.resolve_uniffi_meta_type(ir, cache, &self.builtin, None)?,
            docstring: None,
        })
    }
}
