/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use std::cmp::{max, min};

pub fn pass(en: &mut Enum) -> Result<()> {
    en.is_flat = match en.shape {
        EnumShape::Error { flat } => flat,
        EnumShape::Enum => en.variants.iter().all(|v| v.fields.is_empty()),
    };
    for v in en.variants.iter_mut() {
        v.fields_kind = if v.fields.is_empty() {
            FieldsKind::Unit
        } else if v.fields.iter().any(|f| f.name.is_empty()) {
            FieldsKind::Unnamed
        } else {
            FieldsKind::Named
        };
    }
    determine_discriminants(en)?;
    Ok(())
}

/// Set the `Enum::discr_type` and `Variant::discr` fields
///
/// If we get a value from the metadata, then those will be used.  Otherwise, we will calculate the
/// discriminants by following Rust's logic.
pub fn determine_discriminants(en: &mut Enum) -> Result<()> {
    let signed = match &en.meta_discr_type {
        Some(type_node) => match &type_node.ty {
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 => false,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 => true,
            ty => bail!("Invalid enum discriminant type: {ty:?}"),
        },
        // If not specified, then the discriminant type is signed.  We'll calculate the width as we
        // go through the variant discriminants
        None => true,
    };

    // Calculate all variant discriminants.
    // Use a placeholder value for the type, since we don't necessarily know it yet.
    let mut max_value: u64 = 0;
    let mut min_value: i64 = 0;
    let mut last_variant: Option<&Variant> = None;

    for variant in en.variants.iter_mut() {
        let discr = match &variant.meta_discr {
            None => {
                let lit = match last_variant {
                    None => {
                        if signed {
                            Literal::Int(0, Radix::Decimal, TypeNode::default())
                        } else {
                            Literal::UInt(0, Radix::Decimal, TypeNode::default())
                        }
                    }
                    Some(variant) => match &variant.discr.lit {
                        Literal::UInt(val, _, _) => {
                            Literal::UInt(val + 1, Radix::Decimal, TypeNode::default())
                        }
                        Literal::Int(val, _, _) => {
                            Literal::Int(val + 1, Radix::Decimal, TypeNode::default())
                        }
                        lit => bail!("Invalid enum discriminant literal: {lit:?}"),
                    },
                };
                LiteralNode { lit }
            }
            Some(lit_node) => lit_node.clone(),
        };
        match &discr.lit {
            Literal::UInt(val, _, _) => {
                max_value = max(max_value, *val);
            }
            Literal::Int(val, _, _) => {
                if *val >= 0 {
                    max_value = max(max_value, *val as u64);
                } else {
                    min_value = min(min_value, *val);
                }
            }
            _ => unreachable!(),
        }
        variant.discr = discr;
        last_variant = Some(variant);
    }

    // Finally, we can figure out the discriminant type
    en.discr_type = match &en.meta_discr_type {
        Some(type_node) => type_node.clone(),
        None => {
            if min_value >= i8::MIN as i64 && max_value <= i8::MAX as u64 {
                TypeNode {
                    ty: Type::Int8,
                    ..TypeNode::default()
                }
            } else if min_value >= i16::MIN as i64 && max_value <= i16::MAX as u64 {
                TypeNode {
                    ty: Type::Int16,
                    ..TypeNode::default()
                }
            } else if min_value >= i32::MIN as i64 && max_value <= i32::MAX as u64 {
                TypeNode {
                    ty: Type::Int32,
                    ..TypeNode::default()
                }
            } else if max_value <= i64::MAX as u64 {
                // Note: no need to check `min_value` since that's always in the `i64` bounds.
                TypeNode {
                    ty: Type::Int64,
                    ..TypeNode::default()
                }
            } else {
                bail!("Enum repr not set and magnitude exceeds i64::MAX");
            }
        }
    };
    for variant in en.variants.iter_mut() {
        match &mut variant.discr.lit {
            Literal::UInt(_, _, type_node) | Literal::Int(_, _, type_node) => {
                *type_node = en.discr_type.clone();
            }
            _ => unreachable!(),
        }
    }
    UniffiTraitMethods::fill_from(&mut en.uniffi_trait_methods, &en.uniffi_traits);

    Ok(())
}
