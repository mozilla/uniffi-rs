/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! AST nodes and code to parse them from syn

use proc_macro2::Literal;
// Use IndexMap for mappings.  This preserves the field order and makes the diffs look nicer.
use syn::{Generics, Ident, Type, Visibility};

/// Struct/Enum node in the IR
#[derive(Clone)]
#[allow(dead_code)]
pub struct Node {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub def: NodeDef,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum NodeDef {
    Struct(Struct),
    Enum(Vec<Variant>),
}

pub enum NodeKind<'a> {
    Struct {
        type_name: &'a Ident,
    },
    Variant {
        type_name: &'a Ident,
        variant_name: &'a Ident,
    },
}

#[derive(Clone, Default)]
pub struct Attributes {
    pub from: Option<Ident>,
    pub wraps: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Struct {
    pub fields: Fields,
    pub wraps_field: Option<Field>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Variant {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Fields,
    pub discriminant: Option<Literal>,
}

#[derive(Clone)]
pub enum Fields {
    Unit,
    Named(Vec<Field>),
    Unnamed(Vec<Field>),
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Field {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Option<Ident>,
    pub ty: Type,
    /// Variable name used it patterns, this will be `ident` for named fields and `var{idx}` for
    /// unnamed fields.
    pub var_name: Ident,
}

impl Node {
    pub fn prev_node_ident(&self) -> &Ident {
        self.attrs.from.as_ref().unwrap_or(&self.ident)
    }
}

impl NodeKind<'_> {
    pub fn field_name(&self, field: &Field) -> String {
        match (self, &field.ident) {
            (NodeKind::Struct { .. }, Some(name)) => format!(".{name}"),
            (NodeKind::Struct { .. }, None) => format!(
                ".{}",
                field.var_name.to_string().strip_prefix("var").unwrap()
            ),
            (NodeKind::Variant { variant_name, .. }, Some(name)) => {
                format!(".{variant_name}::{name}")
            }
            (NodeKind::Variant { variant_name, .. }, None) => format!(
                ".{variant_name}::{}",
                field.var_name.to_string().strip_prefix("var").unwrap()
            ),
        }
    }
}

impl Variant {
    pub fn prev_node_ident(&self) -> &Ident {
        self.attrs.from.as_ref().unwrap_or(&self.ident)
    }
}

impl Fields {
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        match self {
            Self::Named(fields) => fields.iter(),
            Self::Unnamed(fields) => fields.iter(),
            Self::Unit => [].iter(),
        }
    }

    pub fn var_names(&self) -> impl Iterator<Item = &Ident> {
        self.iter().map(|f| &f.var_name)
    }

    pub fn not_named(&self) -> bool {
        !matches!(self, Fields::Named(_))
    }
}

impl Field {
    pub fn prev_node_ident(&self) -> Option<&Ident> {
        self.attrs.from.as_ref().or(self.ident.as_ref())
    }
}
