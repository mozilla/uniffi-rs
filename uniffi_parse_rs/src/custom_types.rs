/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::Ident;
use uniffi_meta::CustomTypeMetadata;

use crate::{paths::LookupCache, Ir, RPath, Result};

pub struct CustomType {
    pub docstring: Option<String>,
    pub ident: Ident,
    pub builtin: syn::Type,
}

impl CustomType {
    pub fn custom_type_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        item_path: RPath<'ir>,
    ) -> Result<CustomTypeMetadata> {
        let builtin =
            item_path
                .parent()?
                .resolve_uniffi_meta_type(ir, cache, &self.builtin, None)?;
        let names = item_path.public_path_to_item(ir, cache)?;
        Ok(CustomTypeMetadata {
            module_path: names.module_path,
            name: names.name,
            orig_name: names.orig_name,
            builtin,
            docstring: self.docstring.clone(),
        })
    }
}
