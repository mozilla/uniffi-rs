/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Ident, TypePath};
use uniffi_meta::CustomTypeMetadata;

use crate::{
    macros::CustomTypeMacroCall, paths::LookupCache, trace, types::Type, Error, ErrorKind::*, Ir,
    IrChanges, Item, Module, Namespace, RPath, Result,
};

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

pub fn resolve_custom_type_macro<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module: &Module,
    module_path: &RPath<'ir>,
    macro_call: syn::Macro,
    changes: &mut IrChanges,
) -> Result<()> {
    let mut custom_type = CustomType {
        macro_call: macro_call
            .parse_body()
            .map_err(|e| Error::new_syn(module.source, e))?,
        macro_call_module_path: module_path.syn_path(),
        // set below
        rust_type_ident: None,
    };
    if custom_type.macro_call.remote {
        // Remote types stay where the custom_type! macro was called,
        // for now at least.
        // See https://github.com/mozilla/uniffi-rs/issues/2994
        changes.add(module.id, Item::CustomType(custom_type));
    } else {
        // Non-remote custom types move to the Rust type's module.
        trace!("Finding custom type {}", custom_type.macro_call.ident);
        let rust_type = module_path.resolve_type(
            ir,
            cache,
            &syn::Type::Path(TypePath {
                qself: None,
                path: custom_type.macro_call.ident.clone().into(),
            }),
        )?;
        match rust_type {
            Type::NonUniffi { module_id, ident } => {
                if ident != custom_type.macro_call.ident {
                    // The Rust type has a different name than
                    // the one used in the `custom_type!` call.
                    //
                    // Remember this so type resolution works.
                    custom_type.rust_type_ident = Some(ident);
                }
                changes.add(module_id, Item::CustomType(custom_type));
            }
            _ => {
                return Err(Error::new(
                    module.source,
                    custom_type.macro_call.ident.span(),
                    InvalidCustomType,
                ))
            }
        }
    };
    Ok(())
}
