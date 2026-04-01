/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use proc_macro2::Span;
use syn::{ext::IdentExt, spanned::Spanned, FnArg, Ident, ImplItemFn, Receiver};
use uniffi_meta::ObjectImpl;

use crate::{
    attrs::{ConstructorAttributes, MethodAttributes, ObjectAttributes},
    paths::LookupCache,
    Argument, Error,
    ErrorKind::*,
    Ir, RPath, Result, ReturnType,
};

#[derive(Clone)]
pub struct Object {
    pub attrs: ObjectAttributes,
    pub ident: Ident,
}

#[derive(Clone)]
pub struct Constructor {
    pub attrs: ConstructorAttributes,
    pub is_async: bool,
    pub ident: Ident,
    pub args: Vec<Argument>,
    pub return_type: ReturnType,
}

#[derive(Clone)]
pub struct Method {
    pub attrs: MethodAttributes,
    pub is_async: bool,
    pub ident: Ident,
    pub self_arg: SelfArg,
    pub args: Vec<Argument>,
    pub return_type: ReturnType,
}

#[derive(Clone)]
pub struct SelfArg {
    receiver: Receiver,
}

impl Object {
    pub fn parse(attrs: ObjectAttributes, ident: Ident) -> syn::Result<Self> {
        Ok(Self { attrs, ident })
    }

    pub fn name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| self.ident.unraw().to_string())
    }

    pub fn obj_metadata<'ir>(&self, path: &RPath<'ir>) -> Result<uniffi_meta::ObjectMetadata> {
        Ok(uniffi_meta::ObjectMetadata {
            module_path: path.path_string(),
            name: self.name(),
            remote: self.attrs.remote,
            docstring: self.attrs.docstring.clone(),
            imp: ObjectImpl::Struct,
        })
    }
}

impl Constructor {
    pub fn parse(attrs: ConstructorAttributes, f: ImplItemFn) -> syn::Result<Self> {
        Ok(Self {
            attrs,
            is_async: f.sig.asyncness.is_some(),
            ident: f.sig.ident,
            args: f
                .sig
                .inputs
                .into_iter()
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| self.ident.unraw().to_string())
    }

    pub fn to_constructor_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_name: &str,
        self_ty: &uniffi_meta::Type,
    ) -> Result<uniffi_meta::ConstructorMetadata> {
        let (return_type, throws) =
            self.return_type
                .return_type_and_throws_for_method(ir, cache, module_path, self_ty)?;
        if return_type.as_ref() != Some(self_ty) {
            return Err(Error::new(
                module_path.file_id(),
                self.return_type.return_ty.span(),
                InvalidReturnType,
            ));
        }

        Ok(uniffi_meta::ConstructorMetadata {
            module_path: module_path.path_string(),
            self_name: self_name.to_string(),
            name: self.name(),
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_fn_metadata(ir, cache, module_path, &self.attrs.defaults))
                .collect::<Result<Vec<_>>>()?,
            throws,
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}

impl Method {
    pub fn parse(attrs: MethodAttributes, f: ImplItemFn) -> syn::Result<Self> {
        let mut inputs = f.sig.inputs.into_iter();
        let self_arg = SelfArg::parse(inputs.next(), f.sig.ident.span())?;

        Ok(Self {
            attrs,
            ident: f.sig.ident,
            is_async: f.sig.asyncness.is_some(),
            self_arg,
            args: inputs
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| self.ident.unraw().to_string())
    }

    pub fn to_method_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
        self_name: &str,
        self_ty: &uniffi_meta::Type,
    ) -> Result<uniffi_meta::MethodMetadata> {
        let (return_type, throws) =
            self.return_type
                .return_type_and_throws_for_method(ir, cache, module_path, self_ty)?;

        Ok(uniffi_meta::MethodMetadata {
            module_path: module_path.path_string(),
            self_name: self_name.to_string(),
            name: self.name(),
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            takes_self_by_arc: self.self_arg.takes_self_by_arc(ir, cache, module_path)?,
            inputs: self
                .args
                .iter()
                .map(|arg| {
                    arg.create_method_metadata(
                        ir,
                        cache,
                        module_path,
                        &self.attrs.defaults,
                        self_ty,
                    )
                })
                .collect::<Result<Vec<_>>>()?,
            return_type,
            throws,
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}

impl SelfArg {
    /// Parses sig.inputs.first() into a `SelfArg`
    pub fn parse(arg: Option<FnArg>, ident_span: Span) -> syn::Result<Self> {
        match arg {
            Some(FnArg::Receiver(receiver)) => Ok(Self { receiver }),
            Some(_) => Err(syn::Error::new(arg.span(), InvalidSelfType)),
            None => Err(syn::Error::new(ident_span, MissingSelfType)),
        }
    }

    /// There's no uniffi_meta metadata for the self arg, the only thing we store is the
    /// `takes_self_by_arc` boolean.
    pub fn takes_self_by_arc<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<bool> {
        let self_ty = module_path.resolve_self_type(ir, cache, &self.receiver.ty)?;
        Ok(self_ty.takes_self_by_arc)
    }
}
