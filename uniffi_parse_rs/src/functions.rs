/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, spanned::Spanned, FnArg, Ident, ItemFn, Pat};

use crate::{attrs::FunctionAttributes, paths::LookupCache, ErrorKind::*, Ir, RPath, Result};

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

    pub fn fn_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<uniffi_meta::FnMetadata> {
        let (return_type, throws) =
            self.return_ty
                .return_type_and_throws(ir, cache, module_path)?;

        Ok(uniffi_meta::FnMetadata {
            module_path: module_path.path_string(),
            name: self.name(),
            is_async: self.is_async,
            docstring: self.attrs.docstring.clone(),
            checksum: None,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_fn_metadata(ir, cache, module_path))
                .collect::<Result<Vec<_>>>()?,
            return_type,
            throws,
        })
    }
}

impl Argument {
    pub fn parse(arg: FnArg) -> syn::Result<Self> {
        let span = arg.span();
        let pat_ty = match arg {
            FnArg::Receiver(_) => return Err(syn::Error::new(span, InvalidArgType)),
            FnArg::Typed(pat_ty) => pat_ty,
        };
        let ident = match *pat_ty.pat {
            Pat::Ident(p) => p.ident,
            _ => return Err(syn::Error::new(span, InvalidArgType)),
        };
        Ok(Self {
            ident,
            ty: *pat_ty.ty,
        })
    }

    pub fn create_fn_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        self.create_metadata(ir, cache, path, None)
    }

    pub fn create_method_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        self_type: &uniffi_meta::Type,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        self.create_metadata(ir, cache, path, Some(self_type))
    }

    pub fn create_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<uniffi_meta::FnParamMetadata> {
        let arg = path.resolve_arg(ir, cache, &self.ty, self_ty)?;

        Ok(uniffi_meta::FnParamMetadata {
            name: self.ident.unraw().to_string(),
            ty: arg.ty,
            by_ref: arg.by_ref,
            default: None,
            optional: false,
        })
    }
}

impl ReturnType {
    pub fn parse(return_ty: syn::ReturnType) -> syn::Result<Self> {
        Ok(Self { return_ty })
    }

    pub fn return_type_and_throws<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        self._return_type_and_throws(ir, cache, module_path, None)
    }

    pub fn return_type_and_throws_for_method<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_type: &uniffi_meta::Type,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        self._return_type_and_throws(ir, cache, module_path, Some(self_type))
    }

    fn _return_type_and_throws<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_ty: Option<&uniffi_meta::Type>,
    ) -> Result<(Option<uniffi_meta::Type>, Option<uniffi_meta::Type>)> {
        Ok(match &self.return_ty {
            syn::ReturnType::Default => (None, None),
            syn::ReturnType::Type(_, return_ty) => {
                let rt = module_path.resolve_return_type(ir, cache, return_ty, self_ty)?;
                (rt.ok, rt.err)
            }
        })
    }
}
