/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parsing code for the ast

use quote::format_ident;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    token, Attribute, Ident, Token,
};

use super::ast::*;

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let _: Token![struct] = input.parse()?;
            let ident = input.parse()?;
            let generics = input.parse()?;
            let fields: Fields = input.parse()?;
            if fields.not_named() {
                let _: Token![;] = input.parse()?;
            }
            let mut wraps_field = None;
            for f in fields.iter() {
                if f.attrs.wraps {
                    if wraps_field.is_some() {
                        return Err(syn::Error::new(f.var_name.span(), "Multiple wraps"));
                    } else {
                        wraps_field = Some(f.clone());
                    }
                }
            }
            let st = Struct {
                fields,
                wraps_field,
            };
            Ok(Self {
                attrs,
                vis,
                ident,
                generics,
                def: NodeDef::Struct(st),
            })
        } else if lookahead.peek(Token![enum]) {
            let _: Token![enum] = input.parse()?;
            let ident = input.parse()?;
            let generics = input.parse()?;
            let content;
            braced!(content in input);
            let variants = content.parse_terminated(Variant::parse, Token![,])?;
            Ok(Self {
                attrs,
                vis,
                ident,
                generics,
                def: NodeDef::Enum(variants.into_iter().collect()),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed_attrs = input.call(Attribute::parse_outer)?;
        Self::from_list(&parsed_attrs)
    }
}

impl Attributes {
    pub fn from_list(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Attributes::default();
        for attr in attrs {
            if attr.path().is_ident("node") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("from") {
                        let content;
                        parenthesized!(content in meta.input);
                        result.from = Some(content.parse()?);
                        return Ok(());
                    } else if meta.path.is_ident("wraps") {
                        result.wraps = true;
                        return Ok(());
                    }
                    Err(meta.error("unrecognized node attr"))
                })?;
            }
        }
        Ok(result)
    }
}

impl Parse for Fields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![,]) || lookahead.peek(Token![;]) || lookahead.peek(Token![=]) {
            Ok(Self::Unit)
        } else if lookahead.peek(token::Brace) {
            let content;
            braced!(content in input);
            Ok(Self::Named(
                content
                    .parse_terminated(Field::parse_named, Token![,])?
                    .into_iter()
                    .collect(),
            ))
        } else if lookahead.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let mut counter = 0;
            let mut fields = vec![];
            while !content.is_empty() {
                fields.push(Field::parse_unnamed(&content, counter)?);
                counter += 1;
                if content.is_empty() {
                    break;
                }
                let _: Token![,] = content.parse()?;
            }
            Ok(Self::Unnamed(fields))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let fields = input.parse()?;
        let mut discriminant = None;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            discriminant = Some(input.parse()?);
        }
        Ok(Self {
            attrs,
            vis,
            ident,
            fields,
            discriminant,
        })
    }
}

impl Field {
    fn parse_named(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        let ident: Ident = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self {
            attrs,
            vis,
            ident: Some(ident.clone()),
            ty,
            var_name: ident,
        })
    }

    fn parse_unnamed(input: ParseStream, index: usize) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            ident: None,
            ty: input.parse()?,
            var_name: format_ident!("var{index}"),
        })
    }
}
