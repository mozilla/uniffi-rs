/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, spanned::Spanned, Expr, Ident, Lit};
use uniffi_meta::{EnumShape, LiteralMetadata};

use crate::{
    attrs::{EnumAttributes, FieldAttributes, VariantAttributes},
    ErrorKind::*,
    Field, Result,
};

#[derive(Clone)]
pub struct Enum {
    pub attrs: EnumAttributes,
    pub ident: Ident,
    pub variants: Vec<Variant>,
}

#[derive(Clone)]
pub struct Variant {
    pub attrs: VariantAttributes,
    pub ident: Ident,
    pub discr: Option<LiteralMetadata>,
    pub fields: Vec<Field>,
}

impl Enum {
    pub fn parse(attrs: EnumAttributes, e: syn::ItemEnum) -> syn::Result<Self> {
        let mut variants = vec![];
        for v in e.variants {
            if let Some(variant_attrs) = VariantAttributes::parse(&v.attrs)? {
                variants.push(Variant::parse(&attrs, variant_attrs, v)?);
            }
        }

        Ok(Self {
            attrs,
            ident: e.ident,
            variants,
        })
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn enum_metadata<'ir>(&self) -> Result<uniffi_meta::EnumMetadata> {
        let is_flat = matches!(self.attrs.shape, EnumShape::Error { flat: true });
        Ok(uniffi_meta::EnumMetadata {
            module_path: "".into(), // TODO
            name: self.name(),
            orig_name: None, // TODO
            remote: false,
            non_exhaustive: self.attrs.non_exhaustive,
            discr_type: self.attrs.discr_type.clone(),
            shape: self.attrs.shape,
            docstring: self.attrs.docstring.clone(),
            variants: self
                .variants
                .iter()
                .map(|v| v.create_variant_metadata(is_flat))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl Variant {
    pub fn parse(
        enum_attrs: &EnumAttributes,
        attrs: VariantAttributes,
        v: syn::Variant,
    ) -> syn::Result<Self> {
        let discr = Self::parse_discriminant(&v, enum_attrs)?;
        let mut fields = vec![];
        for f in v.fields {
            if let Some(field_attrs) = FieldAttributes::parse(&f.attrs)? {
                fields.push(Field::parse(field_attrs, f)?);
            }
        }

        Ok(Self {
            attrs,
            ident: v.ident,
            discr,
            fields,
        })
    }

    fn parse_discriminant(
        v: &syn::Variant,
        enum_attrs: &EnumAttributes,
    ) -> syn::Result<Option<LiteralMetadata>> {
        let Some((_, discr)) = v.discriminant.as_ref() else {
            return Ok(None);
        };
        let Some(discr_type) = enum_attrs.discr_type.as_ref() else {
            return Ok(None);
        };
        let signed = matches!(
            discr_type,
            uniffi_meta::Type::Int8
                | uniffi_meta::Type::Int16
                | uniffi_meta::Type::Int32
                | uniffi_meta::Type::Int64
        );

        match discr {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Int(lit) if signed => Ok(Some(LiteralMetadata::Int(
                    lit.base10_parse()?,
                    uniffi_meta::Radix::Decimal,
                    discr_type.clone(),
                ))),
                Lit::Int(lit) => Ok(Some(LiteralMetadata::UInt(
                    lit.base10_parse()?,
                    uniffi_meta::Radix::Decimal,
                    discr_type.clone(),
                ))),
                _ => Err(syn::Error::new(discr.span(), InvalidDiscr)),
            },
            Expr::Unary(expr_unary) if matches!(expr_unary.op, syn::UnOp::Neg(_)) => {
                match &*expr_unary.expr {
                    Expr::Lit(lit) => match &lit.lit {
                        Lit::Int(lit) if signed => Ok(Some(LiteralMetadata::Int(
                            -(lit.base10_parse()?),
                            uniffi_meta::Radix::Decimal,
                            discr_type.clone(),
                        ))),
                        _ => Err(syn::Error::new(discr.span(), InvalidDiscr)),
                    },
                    _ => Err(syn::Error::new(discr.span(), InvalidDiscr)),
                }
            }
            _ => Err(syn::Error::new(discr.span(), InvalidDiscr)),
        }
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn create_variant_metadata<'ir>(&self, flat: bool) -> Result<uniffi_meta::VariantMetadata> {
        Ok(uniffi_meta::VariantMetadata {
            name: self.name(),
            orig_name: None, // TODO
            discr: self.discr.clone(),
            docstring: self.attrs.docstring.clone(),
            fields: if flat {
                vec![]
            } else {
                self.fields
                    .iter()
                    .map(|f| f.create_field_metadata())
                    .collect::<Result<Vec<_>>>()?
            },
        })
    }
}
