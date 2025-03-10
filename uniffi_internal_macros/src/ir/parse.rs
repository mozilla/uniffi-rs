/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parsing code for the ast

use proc_macro2::Span;
use quote::format_ident;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    token, Attribute, Ident, Token,
};

use super::{ast::*, kw};

impl Parse for IrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: kw::name = input.parse()?;
        let _: Token![:] = input.parse()?;
        let name = input.parse()?;
        let _: Token![;] = input.parse()?;
        let items = input.parse()?;

        Ok(Self { name, items })
    }
}

impl Parse for DefineIrPassInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        let from = content.parse()?;
        let to = content.parse()?;

        let content2;
        braced!(content2 in input);
        let from_items = content2.parse()?;

        let content3;
        braced!(content3 in input);
        let to_items = content3.parse()?;

        Ok(Self {
            from,
            from_items,
            to,
            to_items,
        })
    }
}

impl Parse for ConstructNodeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let fields = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();
        let expr = input.parse()?;
        Ok(Self { fields, expr })
    }
}

impl Parse for Items {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Self::default();
        while !input.is_empty() {
            let attrs = input.parse()?;
            if input.peek(Token![impl]) {
                items.impls.push(input.parse()?);
            } else {
                let node = Node::parse_with_attrs(attrs, input)?;
                items.nodes.insert(node.ident.clone(), node);
            }
        }
        Ok(items)
    }
}

impl Node {
    /// Parse a node, with attributes that have already been parsed
    fn parse_with_attrs(attrs: Attributes, input: ParseStream) -> syn::Result<Self> {
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
            Ok(Self {
                attrs,
                vis,
                ident,
                generics,
                def: NodeDef::Struct(fields),
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
                def: NodeDef::Enum(Variants {
                    variants: variants.into_iter().map(|v| (v.ident.clone(), v)).collect(),
                }),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        Node::parse_with_attrs(attrs, input)
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
            if attr.path().is_ident("from_uniffi_meta") {
                result.from_uniffi_meta = Some(attr.parse_args()?);
            } else if attr.path().is_ident("pass_only") {
                result.pass_only = true;
            } else {
                result.other.push(attr.clone());
            }
        }
        Ok(result)
    }
}

impl Parse for Generics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.peek(Token![<]) {
            return Ok(Self::default());
        }
        let lt_token = input.parse()?;
        let mut ty_params = vec![];
        loop {
            if input.peek(Token![>]) {
                break;
            }
            ty_params.push(input.parse()?);
            if input.peek(Token![>]) {
                break;
            }
            let _: Token![,] = input.parse()?;
        }
        let gt_token = input.parse()?;
        Ok(Self {
            lt_token: Some(lt_token),
            ty_params,
            gt_token: Some(gt_token),
        })
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
                    .parse_terminated(Field::parse, Token![,])?
                    .into_iter()
                    .map(|f| (f.ident.clone(), f))
                    .collect(),
            ))
        } else if lookahead.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let mut counter = 0;
            let mut fields = vec![];
            while !content.is_empty() {
                fields.push(TupleField::parse(&content, counter)?);
                counter += 1;
                if content.is_empty() {
                    break;
                }
                let _: Token![,] = content.parse()?;
            }
            Ok(Self::Tuple(fields))
        } else {
            Err(lookahead.error())
        }
    }
}

impl TupleField {
    fn parse(input: ParseStream, index: usize) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            ty: input.parse()?,
            var_name: format_ident!("var{index}"),
        })
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
            which_irs: Irs::Default,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse()?;
        let vis = input.parse()?;
        let ident = input.parse()?;
        let _ = input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self {
            attrs,
            vis,
            ident,
            ty,
            which_irs: Irs::Default,
        })
    }
}

impl Parse for IrPassInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut from = None;
        let mut to = None;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::from) {
                let kw: kw::from = input.parse()?;
                if from.is_some() {
                    return Err(syn::Error::new(kw.span, "Duplicate from"));
                }
                let _: Token![:] = input.parse()?;
                from = Some(input.parse()?);
            } else if lookahead.peek(kw::to) {
                let kw: kw::to = input.parse()?;
                if to.is_some() {
                    return Err(syn::Error::new(kw.span, "Duplicate to"));
                }
                let _: Token![:] = input.parse()?;
                to = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
            let _: Token![;] = input.parse()?;
        }
        Ok(Self {
            from: from.ok_or_else(|| syn::Error::new(Span::call_site(), "missing from"))?,
            to: to.ok_or_else(|| syn::Error::new(Span::call_site(), "missing to"))?,
        })
    }
}
