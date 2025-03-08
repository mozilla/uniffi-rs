/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use indexmap::IndexMap;
use syn::Ident;

use super::ast::*;

impl Node {
    /// Create a merged node that combines the fields in the `from` and `to`
    ///
    /// If successful, this returns an Ok value with them merged node.  We will use that to define
    /// the node in the pass IR and derive `FromNode` implementations for it.
    ///
    /// If not, this returns an Err value with `self`.  We will use that for the pass IR and
    /// `FromNode` will need to be defined manually.
    pub fn try_merge(self, from_node: Self) -> Result<Self, Self> {
        if !self.can_merge(&from_node) {
            return Err(self);
        }

        let def = match (self.def, from_node.def) {
            (NodeDef::Enum(variants), NodeDef::Enum(from_variants)) => {
                NodeDef::Enum(variants.merge(from_variants))
            }
            (NodeDef::Struct(fields), NodeDef::Struct(from_fields)) => {
                NodeDef::Struct(fields.merge(from_fields))
            }
            _ => unreachable!("should not be possible if `can_merge` returns true"),
        };
        Ok(Node {
            attrs: self.attrs.clone(),
            vis: self.vis.clone(),
            ident: self.ident.clone(),
            def,
        })
    }

    fn can_merge(&self, from_node: &Self) -> bool {
        match (&self.def, &from_node.def) {
            (NodeDef::Struct(fields), NodeDef::Struct(from_fields)) => {
                fields.can_merge(from_fields)
            }
            (NodeDef::Enum(variants), NodeDef::Enum(from_variants)) => {
                variants.can_merge(from_variants)
            }
            _ => false,
        }
    }
}

impl Variants {
    fn can_merge(&self, from_variants: &Self) -> bool {
        for (ident, variant) in self.variants.iter() {
            if let Some(from_variant) = from_variants.variants.get(ident) {
                if !variant.fields.can_merge(&from_variant.fields) {
                    return false;
                }
            }
        }
        true
    }

    fn merge(mut self, from_variants: Self) -> Self {
        let mut merged: IndexMap<Ident, Variant> = from_variants
            .variants
            .into_iter()
            .map(|(ident, mut from_variant)| {
                match self.variants.shift_remove(&ident) {
                    Some(to_variant) => {
                        from_variant.fields = to_variant.fields.merge(from_variant.fields);
                    }
                    None => {
                        from_variant.which_irs = Irs::FromOnly;
                    }
                };
                (ident, from_variant)
            })
            .collect();
        // Anything left in `variants` is only in the `to` IR
        for v in self.variants.values_mut() {
            v.which_irs = Irs::ToOnly;
        }
        merged.extend(self.variants);
        Self { variants: merged }
    }
}

impl Fields {
    fn can_merge(&self, from_fields: &Self) -> bool {
        match (self, from_fields) {
            (Self::Unit, Self::Unit) => true,
            (Self::Named(_), Self::Named(_)) => true,
            (Self::Tuple(types), Self::Tuple(to_types)) => {
                if types.len() != to_types.len() {
                    false
                } else {
                    types
                        .iter()
                        .map(|ty| &ty.ty)
                        .eq(to_types.iter().map(|ty| &ty.ty))
                }
            }
            _ => false,
        }
    }

    fn merge(self, from_fields: Self) -> Self {
        match (self, from_fields) {
            (Fields::Unit, Fields::Unit) => Fields::Unit,
            (Fields::Named(mut to_fields), Fields::Named(from_fields)) => {
                let mut merged = IndexMap::<Ident, Field>::default();
                for (ident, mut field) in from_fields {
                    match to_fields.shift_remove(&ident) {
                        None => {
                            field.which_irs = Irs::FromOnly;
                            merged.insert(ident, field);
                        }
                        Some(mut to_field) => {
                            // Need to use the `Ident` from the original field or else we'll get
                            // name errors
                            to_field.ident = field.ident;
                            merged.insert(ident, to_field);
                        }
                    }
                }
                // anything left in `fields` is only in the `to` Ir
                for f in to_fields.values_mut() {
                    f.which_irs = Irs::ToOnly;
                }
                merged.extend(to_fields);
                Fields::Named(merged)
            }
            (Fields::Tuple(types), Fields::Tuple(_)) => Fields::Tuple(types),
            _ => unreachable!("should not be possible if `can_merge` returns true"),
        }
    }
}
