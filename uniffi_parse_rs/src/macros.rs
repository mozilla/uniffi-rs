/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Attribute, ExprClosure, Ident, ItemMacro, LitStr, Macro, Token,
};

use crate::{
    attrs::extract_docstring, kw, paths::LookupCache, BuiltinItem, Ir, Item, Namespace, RPath,
};

/// Try resolving Item::Macro into a more specific item like Item::UseRemoteType
pub fn maybe_resolve_macro<'ir>(
    ir: &'ir Ir,
    cache: &mut LookupCache<'ir>,
    path: &RPath<'ir>,
    mac: &ItemMacro,
) -> syn::Result<Option<Item>> {
    let builtin = match path.resolve(ir, cache, &mac.mac.path, Namespace::Macro) {
        // Ignore errors, maybe the macro comes from an unparsed crate.
        Err(_) => return Ok(None),
        Ok(path) => match path.item() {
            Ok(Item::Builtin(b)) => b,
            _ => return Ok(None),
        },
    };
    match builtin {
        // Note: custom_newtype and custom_type share enough of the same syntax that we can use the
        // same parser for both
        BuiltinItem::UniffiMacro("custom_type") | BuiltinItem::UniffiMacro("custom_newtype") => {
            Ok(Some(Item::CustomTypeMacroCall(mac.mac.parse_body()?)))
        }
        BuiltinItem::UniffiMacro("use_remote_type") => {
            Ok(Some(Item::UseRemoteType(mac.mac.parse_body()?)))
        }
        _ => Ok(None),
    }
}

/// Parsed custom_type! macro call
pub struct CustomTypeMacroCall {
    pub remote: bool,
    pub docstring: Option<String>,
    pub ident: Ident,
    pub bridge_type: syn::Type,
}

impl Parse for CustomTypeMacroCall {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut docstring = None;
        let mut remote = false;
        let attrs = input.call(Attribute::parse_outer)?;
        for attr in attrs.iter() {
            extract_docstring(&mut docstring, &attr.meta);
        }
        // Parse the custom / UniFFI type which are both required
        let ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let bridge_type = input.parse()?;
        // If there's an extra arg with a brace, just skip over it.  It's only used by the
        // Rust proc-macros.
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            if input.peek(Brace) {
                let content;
                braced!(content in input);
                let items = content.parse_terminated(CustomTypeBodyItem::parse, Token![,])?;
                remote = items
                    .iter()
                    .any(|i| matches!(i, CustomTypeBodyItem::Remote));
            }
        };
        Ok(Self {
            remote,
            docstring,
            ident,
            bridge_type,
        })
    }
}

/// Item inside the braces of a custom_type! macro
///
/// This represents a single item inside the braces
#[allow(dead_code)]
enum CustomTypeBodyItem {
    Remote,
    TryLift(ExprClosure),
    Lower(ExprClosure),
}

impl Parse for CustomTypeBodyItem {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::remote) {
            input.parse::<kw::remote>()?;
            Ok(Self::Remote)
        } else if lookahead.peek(kw::try_lift) {
            let _: kw::try_lift = input.parse()?;
            let _: Token![:] = input.parse()?;
            Ok(Self::TryLift(input.parse()?))
        } else if lookahead.peek(kw::lower) {
            let _: kw::lower = input.parse()?;
            let _: Token![:] = input.parse()?;
            Ok(Self::Lower(input.parse()?))
        } else {
            Err(lookahead.error())
        }
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
