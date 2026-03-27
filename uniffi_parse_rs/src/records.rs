/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{ext::IdentExt, Ident, ItemStruct};

use crate::{
    attrs::{FieldAttributes, RecordAttributes},
    paths::LookupCache,
    Ir, RPath, Result,
};

#[derive(Clone)]
pub struct Record {
    pub attrs: RecordAttributes,
    pub ident: Ident,
    pub fields: Vec<Field>,
}

#[derive(Clone)]
pub struct Field {
    pub attrs: FieldAttributes,
    pub ident: Option<Ident>,
    pub ty: syn::Type,
}

impl Record {
    pub fn parse(attrs: RecordAttributes, st: ItemStruct) -> syn::Result<Self> {
        let mut fields = vec![];
        for f in st.fields {
            if let Some(field_attrs) = FieldAttributes::parse(&f.attrs)? {
                fields.push(Field::parse(field_attrs, f)?);
            }
        }
        Ok(Self {
            attrs,
            ident: st.ident,
            fields,
        })
    }

    pub fn name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| self.ident.unraw().to_string())
    }

    pub fn record_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        path: &RPath<'ir>,
    ) -> Result<uniffi_meta::RecordMetadata> {
        Ok(uniffi_meta::RecordMetadata {
            module_path: path.path_string(),
            name: self.name(),
            remote: self.attrs.remote,
            docstring: self.attrs.docstring.clone(),
            fields: self
                .fields
                .iter()
                .map(|f| f.create_field_metadata(ir, cache, path))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl Field {
    pub fn parse(attrs: FieldAttributes, f: syn::Field) -> syn::Result<Self> {
        Ok(Self {
            attrs,
            ident: f.ident,
            ty: f.ty,
        })
    }

    pub fn name(&self) -> String {
        self.attrs
            .name
            .clone()
            .unwrap_or_else(|| match &self.ident {
                Some(i) => i.unraw().to_string(),
                None => "".to_string(),
            })
    }

    pub fn create_field_metadata<'ir>(
        &self,
        ir: &'ir Ir,
        cache: &mut LookupCache<'ir>,
        module_path: &RPath<'ir>,
    ) -> Result<uniffi_meta::FieldMetadata> {
        let name = self.name();
        let ty = module_path.resolve_uniffi_meta_type(ir, cache, &self.ty, None)?;
        let default = self
            .attrs
            .default
            .as_ref()
            .map(|d| d.create_default_value_metadata(module_path.file_id(), &ty))
            .transpose()?;

        Ok(uniffi_meta::FieldMetadata {
            name,
            ty,
            default,
            docstring: self.attrs.docstring.clone(),
        })
    }
}
