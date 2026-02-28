/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, spanned::Spanned, FnArg, Ident, ItemFn, Pat};

use crate::{attrs::FunctionAttributes, Result};

#[derive(Clone)]
pub struct Function {
    pub attrs: FunctionAttributes,
    pub is_async: bool,
    pub ident: Ident,
    pub args: Vec<Argument>,
    pub return_ty: ReturnType,
}

#[derive(Clone)]
pub struct Argument {
    pub ident: Ident,
    pub ty: syn::Type,
}

#[derive(Clone)]
pub struct ReturnType {
    pub return_ty: syn::ReturnType,
}

impl Function {
    pub fn parse(attrs: FunctionAttributes, f: ItemFn) -> syn::Result<Self> {
        Ok(Self {
            attrs,
            ident: f.sig.ident,
            is_async: f.sig.asyncness.is_some(),
            args: f
                .sig
                .inputs
                .into_iter()
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_ty: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn fn_metadata<'ir>(&self) -> Result<uniffi_meta::FnMetadata> {
        Ok(uniffi_meta::FnMetadata {
            module_path: "".into(), // TODO
            name: self.name(),
            is_async: self.is_async,
            docstring: self.attrs.docstring.clone(),
            checksum: None,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_metadata())
                .collect::<Result<Vec<_>>>()?,
            return_type: None, // TODO
            throws: None,      // TODO
        })
    }
}

impl Argument {
    pub fn parse(arg: FnArg) -> syn::Result<Self> {
        let span = arg.span();
        let pat_ty = match arg {
            FnArg::Receiver(_) => return Err(syn::Error::new(span, "invalid arg type")),
            FnArg::Typed(pat_ty) => pat_ty,
        };
        let ident = match *pat_ty.pat {
            Pat::Ident(p) => p.ident,
            _ => return Err(syn::Error::new(span, "invalid arg type")),
        };
        Ok(Self {
            ident,
            ty: *pat_ty.ty,
        })
    }

    pub fn create_metadata<'ir>(&self) -> Result<uniffi_meta::FnParamMetadata> {
        Ok(uniffi_meta::FnParamMetadata {
            name: self.ident.unraw().to_string(),
            ty: uniffi_meta::Type::Int8, // TODO
            by_ref: false,
            default: None,
            optional: false,
        })
    }
}

impl ReturnType {
    pub fn parse(return_ty: syn::ReturnType) -> syn::Result<Self> {
        Ok(Self { return_ty })
    }

    pub fn return_type_and_throws(
        &self,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        Ok((None, None)) // TODO
    }
}
