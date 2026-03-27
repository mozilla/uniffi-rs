/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    bracketed,
    meta::ParseNestedMeta,
    parenthesized,
    parse::{Nothing, ParseStream},
    token::Bracket,
    Ident, Lit,
};
use uniffi_meta::{DefaultValueMetadata, LiteralMetadata, Radix, Type};

use crate::{files::FileId, kw, Error, ErrorKind::*, Result};

#[derive(Clone)]
pub struct Default {
    kind: DefaultKind,
    span: Span,
}

#[derive(Clone)]
pub enum DefaultKind {
    /// Type-specific default value
    Default,
    EmptySequence,
    None,
    Some(Box<Default>),
    Lit(Lit),
}

#[derive(Clone, Default)]
pub struct DefaultMap {
    defaults: HashMap<Ident, Default>,
}

impl Default {
    /// Parse a default value
    ///
    /// This parses everything after `meta.path`
    pub fn parse(meta: ParseNestedMeta) -> syn::Result<Self> {
        Ok(Self {
            span: meta.input.span(),
            kind: DefaultKind::parse(meta)?,
        })
    }

    pub fn create_default_value_metadata(
        &self,
        source: FileId,
        ty: &Type,
    ) -> Result<DefaultValueMetadata> {
        Ok(match &self.kind {
            DefaultKind::Default => DefaultValueMetadata::Default,
            DefaultKind::EmptySequence => {
                DefaultValueMetadata::Literal(LiteralMetadata::EmptySequence)
            }
            DefaultKind::None => DefaultValueMetadata::Literal(LiteralMetadata::None),
            DefaultKind::Some(inner) => {
                let Type::Optional { inner_type } = ty else {
                    return Err(Error::new(source, self.span, InvalidDefault));
                };
                DefaultValueMetadata::Literal(LiteralMetadata::Some {
                    inner: Box::new(inner.create_default_value_metadata(source, inner_type)?),
                })
            }
            DefaultKind::Lit(lit) => {
                DefaultValueMetadata::Literal(self.lit_into_uniffi_meta(source, lit, ty)?)
            }
        })
    }

    fn lit_into_uniffi_meta(
        &self,
        source: FileId,
        lit: &Lit,
        ty: &Type,
    ) -> Result<LiteralMetadata> {
        match lit {
            Lit::Int(i) => match ty {
                Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => Ok(LiteralMetadata::Int(
                    i.base10_parse().map_err(|e| Error::new_syn(source, e))?,
                    Radix::Decimal,
                    ty.clone(),
                )),
                Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 => {
                    Ok(LiteralMetadata::UInt(
                        i.base10_parse().map_err(|e| Error::new_syn(source, e))?,
                        Radix::Decimal,
                        ty.clone(),
                    ))
                }
                _ => Err(Error::new(source, lit.span(), InvalidDefault)),
            },
            Lit::Float(f) => match ty {
                Type::Float32 | Type::Float64 => Ok(LiteralMetadata::Float(
                    f.base10_parse().map_err(|e| Error::new_syn(source, e))?,
                    ty.clone(),
                )),
                _ => Err(Error::new(source, lit.span(), InvalidDefault)),
            },
            Lit::Str(s) if *ty == Type::String => Ok(LiteralMetadata::String(s.value())),
            Lit::Bool(b) if *ty == Type::Boolean => Ok(LiteralMetadata::Boolean(b.value)),
            _ => Err(Error::new(source, lit.span(), InvalidDefault)),
        }
    }
}

impl DefaultKind {
    fn parse(meta: ParseNestedMeta) -> syn::Result<Self> {
        if meta.input.is_empty() {
            Ok(Self::Default)
        } else {
            meta.value()?;
            Self::parse_value(meta.input)
        }
    }

    fn parse_value(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Bracket) {
            let content;
            let _ = bracketed!(content in input);
            content.parse::<Nothing>()?;
            Ok(Self::EmptySequence)
        } else if input.peek(kw::None) {
            let _: kw::None = input.parse()?;
            Ok(Self::None)
        } else if input.peek(kw::Some) {
            let _: kw::Some = input.parse()?;
            let content;
            let _ = parenthesized!(content in input);
            Ok(Self::Some(Box::new(Default {
                span: content.span(),
                kind: Self::parse_value(&content)?,
            })))
        } else {
            Ok(Self::Lit(input.parse()?))
        }
    }
}

impl DefaultMap {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        meta.parse_nested_meta(|meta| {
            let ident = meta.path.require_ident()?;
            if self.defaults.contains_key(ident) {
                return Err(meta.error("Multiple default values for {ident}"));
            }
            self.defaults.insert(ident.clone(), Default::parse(meta)?);
            Ok(())
        })
    }

    pub fn get_uniffi_meta(
        &self,
        source: FileId,
        ident: &Ident,
        ty: &Type,
    ) -> Result<Option<DefaultValueMetadata>> {
        match self.defaults.get(ident) {
            None => Ok(None),
            Some(d) => Ok(Some(d.create_default_value_metadata(source, ty)?)),
        }
    }
}
