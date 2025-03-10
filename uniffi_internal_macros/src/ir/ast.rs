/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! AST nodes and code to parse them from syn

use proc_macro2::Literal;
// Use IndexMap for mappings.  This preserves the field order and makes the diffs look nicer.
use indexmap::IndexMap;
use syn::{Attribute, ExprStruct, Ident, ItemImpl, Path, Token, Type, TypeParam, Visibility};

pub struct DefineIrPassInput {
    pub from: Path,
    pub from_items: Items,
    pub to: Path,
    pub to_items: Items,
}

pub struct IrInput {
    pub name: Ident,
    pub items: Items,
}

pub struct IrPassInput {
    pub from: Path,
    pub to: Path,
}

pub struct ConstructNodeInput {
    pub fields: Vec<Ident>,
    pub expr: ExprStruct,
}

#[derive(Default, Clone)]
pub struct Items {
    pub nodes: IndexMap<Ident, Node>,
    pub impls: Vec<ItemImpl>,
}

/// Struct/Enum node in the IR
#[derive(Clone)]
pub struct Node {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub def: NodeDef,
}

#[derive(Clone, Default)]
pub struct Generics {
    pub lt_token: Option<Token![<]>,
    pub ty_params: Vec<TypeParam>,
    pub gt_token: Option<Token![>]>,
}

#[derive(Clone)]
pub enum NodeDef {
    Struct(Fields),
    Enum(Variants),
}

#[derive(Clone, Default)]
pub struct Attributes {
    pub from_uniffi_meta: Option<Ident>,
    pub pass_only: bool,
    pub other: Vec<Attribute>,
}

#[derive(Clone)]
pub struct Variants {
    pub variants: IndexMap<Ident, Variant>,
}

#[derive(Clone)]
pub struct Variant {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Ident,
    pub fields: Fields,
    pub discriminant: Option<Literal>,
    pub which_irs: Irs,
}

#[derive(Clone)]
pub enum Fields {
    Unit,
    Named(IndexMap<Ident, Field>),
    Tuple(Vec<TupleField>),
}

#[derive(Clone)]
pub struct Field {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ident: Ident,
    pub ty: Type,
    pub which_irs: Irs,
}

#[derive(Clone)]
pub struct TupleField {
    pub attrs: Attributes,
    pub vis: Visibility,
    pub ty: Type,
    /// Variable name to use when unpacking values from the tuple.
    /// These have the form `var{index}`
    pub var_name: Ident,
}

impl Fields {
    pub fn not_named(&self) -> bool {
        !matches!(self, Fields::Named(_))
    }
}

/// Tracks fields/variants that only appear in one IR,
#[derive(Clone)]
pub enum Irs {
    // Appears in all IRs, or this field/variant belongs to an unmerged node.
    Default,
    FromOnly,
    ToOnly,
}

impl Variant {
    pub fn in_from_ir(&self) -> bool {
        !self.attrs.pass_only && matches!(self.which_irs, Irs::Default | Irs::FromOnly)
    }

    pub fn in_to_ir(&self) -> bool {
        !self.attrs.pass_only && matches!(self.which_irs, Irs::Default | Irs::ToOnly)
    }
}

impl Field {
    pub fn in_from_ir(&self) -> bool {
        !self.attrs.pass_only && matches!(self.which_irs, Irs::Default | Irs::FromOnly)
    }

    pub fn in_to_ir(&self) -> bool {
        !self.attrs.pass_only && matches!(self.which_irs, Irs::Default | Irs::ToOnly)
    }
}
