/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::Ident;
use uniffi_meta::CustomTypeMetadata;

use crate::{macros::CustomTypeMacroCall, paths::LookupCache, Ir, Namespace, RPath, Result};

pub struct CustomType {
    pub macro_call: CustomTypeMacroCall,
    // Full path to module with the macro call, used to resolve the bridge type
    // This is needed because the CustomType item is stored in the module where the Rust type lives.
    pub macro_call_module_path: syn::Path,
    // Ident of the actual Rust type, if it's different than the `ident` of the custom type itself.
    pub rust_type_ident: Option<Ident>,
}

impl CustomType {
    pub fn custom_type_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        item_path: RPath<'ir>,
    ) -> Result<CustomTypeMetadata> {
        let macro_module_path = item_path.crate_root()?.resolve(
            ir,
            cache,
            &self.macro_call_module_path,
            Namespace::Type,
        )?;
        let builtin = macro_module_path.resolve_uniffi_meta_type(
            ir,
            cache,
            &self.macro_call.bridge_type,
            None,
        )?;
        let names = item_path.public_path_to_item(ir, cache)?;

        Ok(CustomTypeMetadata {
            module_path: names.module_path,
            name: names.name,
            orig_name: names.orig_name,
            builtin,
            docstring: self.macro_call.docstring.clone(),
        })
    }

    pub fn rust_type_ident(&self) -> &Ident {
        self.rust_type_ident
            .as_ref()
            .unwrap_or(&self.macro_call.ident)
    }
}
