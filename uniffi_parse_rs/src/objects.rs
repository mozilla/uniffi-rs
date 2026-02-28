/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, Ident, ImplItemFn};
use uniffi_meta::ObjectImpl;

use crate::{
    attrs::{ConstructorAttributes, MethodAttributes, ObjectAttributes},
    Argument, Result, ReturnType,
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
    pub args: Vec<Argument>,
    pub return_type: ReturnType,
}

impl Object {
    pub fn parse(attrs: ObjectAttributes, ident: Ident) -> syn::Result<Self> {
        Ok(Self { attrs, ident })
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn obj_metadata(&self) -> Result<uniffi_meta::ObjectMetadata> {
        Ok(uniffi_meta::ObjectMetadata {
            module_path: "".into(), // TODO
            name: self.name(),
            remote: false,
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
        self.ident.unraw().to_string()
    }

    pub fn to_constructor_metadata(
        &self,
        self_name: &str,
    ) -> Result<uniffi_meta::ConstructorMetadata> {
        Ok(uniffi_meta::ConstructorMetadata {
            module_path: "".into(), // TODO
            self_name: self_name.to_string(),
            name: self.name(),
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_metadata())
                .collect::<Result<Vec<_>>>()?,
            throws: None, // TODO
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}

impl Method {
    pub fn parse(attrs: MethodAttributes, f: ImplItemFn) -> syn::Result<Self> {
        let inputs = f.sig.inputs.into_iter().take(1);

        Ok(Self {
            attrs,
            ident: f.sig.ident,
            is_async: f.sig.asyncness.is_some(),
            args: inputs
                .map(Argument::parse)
                .collect::<syn::Result<Vec<_>>>()?,
            return_type: ReturnType::parse(f.sig.output)?,
        })
    }

    pub fn name(&self) -> String {
        self.ident.unraw().to_string()
    }

    pub fn to_method_metadata(&self, self_name: &str) -> Result<uniffi_meta::MethodMetadata> {
        Ok(uniffi_meta::MethodMetadata {
            module_path: "".into(), // TODO
            self_name: self_name.to_string(),
            name: self.name(),
            docstring: self.attrs.docstring.clone(),
            is_async: self.is_async,
            takes_self_by_arc: false,
            inputs: self
                .args
                .iter()
                .map(|arg| arg.create_metadata())
                .collect::<Result<Vec<_>>>()?,
            return_type: None, // TODO
            throws: None,      // TODO
            // Method checksums are not supported, we can implement an improved system by
            // checksumming the entire interface and having a single checksum
            checksum: None,
        })
    }
}
