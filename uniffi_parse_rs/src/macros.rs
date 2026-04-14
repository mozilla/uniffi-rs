/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Attribute, Ident, ItemMacro, LitStr, Macro, Token,
};

use crate::{
    attrs::extract_docstring, paths::LookupCache, BuiltinItem, CustomType, Error, Ir, Item, Module,
    Namespace, RPath, Result,
};

/// Resolve Item::Macro to more specific items like Item::UseRemoteType
///
/// This needs to run after adding all crates parsing, since it requires looking up module paths.
pub fn resolve_macros(ir: &mut Ir) -> Result<()> {
    let mut cache = LookupCache::default();
    let mut found_items = HashMap::<usize, Vec<Item>>::new();
    for (mut path, module) in ir.crate_roots_and_paths() {
        resolve_macros_recurse(ir, &mut cache, module, &mut path, &mut found_items)?;
    }
    for module in ir.crate_roots_mut() {
        insert_found_items(module, &mut found_items);
    }

    Ok(())
}

fn resolve_macros_recurse<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    module: &'ir Module,
    path: &mut RPath<'ir>,
    found_items: &mut HashMap<usize, Vec<Item>>,
) -> Result<()> {
    for item in module.items.iter() {
        match item {
            Item::Macro(mac) => {
                if let Some(resolved) = maybe_resolve_macro(ir, cache, path, mac)? {
                    found_items.entry(module.id).or_default().push(resolved);
                }
            }
            Item::Module(child) => {
                path.push(item);
                resolve_macros_recurse(ir, cache, child, path, found_items)?;
                path.pop();
            }
            _ => (),
        }
    }
    Ok(())
}

/// Try resolving Item::Macro into a more specific item like Item::UseRemoteType
fn maybe_resolve_macro<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    path: &RPath<'ir>,
    mac: &ItemMacro,
) -> Result<Option<Item>> {
    let builtin = match path.resolve(ir, cache, &mac.mac.path, Namespace::Macro) {
        // Ignore errors, maybe the macro comes from an unparsed crate.
        Err(_) => return Ok(None),
        Ok(path) => match path.item()? {
            Item::Builtin(b) => b,
            _ => return Ok(None),
        },
    };
    match builtin {
        // Note: custom_newtype and custom_type share enough of the same syntax that we can use the
        // same parser for both
        BuiltinItem::UniffiMacro("custom_type") | BuiltinItem::UniffiMacro("custom_newtype") => {
            let args: CustomTypeArgs = mac
                .mac
                .parse_body()
                .map_err(|e| Error::new_syn(path.file_id(), e))?;
            Ok(Some(Item::CustomType(CustomType {
                docstring: args.docstring,
                ident: args.ident,
                builtin: args.builtin,
            })))
        }
        BuiltinItem::UniffiMacro("use_remote_type") => Ok(Some(Item::UseRemoteType(
            mac.mac
                .parse_body()
                .map_err(|e| Error::new_syn(path.file_id(), e))?,
        ))),
        _ => Ok(None),
    }
}

fn insert_found_items(module: &mut Module, items: &mut HashMap<usize, Vec<Item>>) {
    if let Some(found_items) = items.remove(&module.id) {
        module.items.extend(found_items);
    }
    for item in module.items.iter_mut() {
        if let Item::Module(child) = item {
            insert_found_items(child, items);
        }
    }
}

struct CustomTypeArgs {
    docstring: Option<String>,
    ident: Ident,
    builtin: syn::Type,
}

impl Parse for CustomTypeArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut docstring = None;
        let attrs = input.call(Attribute::parse_outer)?;
        for attr in attrs.iter() {
            extract_docstring(&mut docstring, &attr.meta);
        }
        // Parse the custom / UniFFI type which are both required
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let builtin = input.parse()?;
        // If there's an extra arg with a brace, just skip over it.  It's only used by the
        // Rust proc-macros.
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.peek(Brace) {
                let content;
                braced!(content in input);
                let _tokens: TokenStream = content.parse()?;
            }
        };
        Ok(Self {
            docstring,
            ident,
            builtin,
        })
    }
}

/// Try to parse the `include_scaffolding!` macro.
///
/// This is done earlier than all other macros because we need to know the UDL name early so that
/// we can load items from there.
///
/// This means the detection is a bit worse and won't handle renames, but that seems okay for now.
pub fn parse_include_scaffolding(mac: &Macro) -> syn::Result<Option<LitStr>> {
    match mac.path.segments.last() {
        Some(s) if s.ident == "include_scaffolding" => Ok(Some(mac.parse_body()?)),
        _ => Ok(None),
    }
}
