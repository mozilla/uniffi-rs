/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp::{max, min};

use super::*;

pub fn map_variants(
    discr_type: &Option<Type>,
    variants: Vec<initial::Variant>,
    context: &Context,
) -> Result<Vec<Variant>> {
    let discr_info = determine_discriminants(discr_type, &variants, context)?;
    if discr_info.variant_discrs.len() != variants.len() {
        bail!("Enum::map_node() variant/discriminant length mismatch");
    }

    variants
        .into_iter()
        .zip(discr_info.variant_discrs)
        .map(move |(v, discr)| {
            let mut child_context = context.clone();
            let context = &mut child_context;
            context.update_from_variant(&v)?;

            Ok(Variant {
                fields_kind: records::fields_kind(&v.fields),
                discr,
                name: rename::variant(v.name, context)?,
                fields: v.fields.map_node(context)?,
                docstring: v.docstring,
            })
        })
        .collect()
}

pub fn discr_type(en: &initial::Enum, context: &Context) -> Result<TypeNode> {
    Ok(determine_discriminants(&en.discr_type, &en.variants, context)?.discr_type)
}

pub fn is_flat(en: &initial::Enum) -> bool {
    match en.shape {
        EnumShape::Error { flat } => flat,
        EnumShape::Enum => en.variants.iter().all(|v| v.fields.is_empty()),
    }
}

pub struct DiscriminantInfo {
    pub discr_type: TypeNode,
    pub variant_discrs: Vec<Literal>,
}

/// Set the `Enum::discr_type` and `Variant::discr` fields
///
/// If we get a value from the metadata, then those will be used.  Otherwise, we will calculate the
/// discriminants by following Rust's logic.
pub fn determine_discriminants(
    discr_type: &Option<Type>,
    variants: &[initial::Variant],
    context: &Context,
) -> Result<DiscriminantInfo> {
    let signed = match discr_type {
        Some(ty) => match ty {
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
    let placeholder_type_node: TypeNode = Type::UInt8.map_node(context)?;
    let mut max_value: u64 = 0;
    let mut min_value: i64 = 0;
    let mut variant_discrs: Vec<Literal> = vec![];

    for variant in variants.iter() {
        let discr = match &variant.discr {
            None => match variant_discrs.last() {
                None => {
                    if signed {
                        Literal::Int(0, Radix::Decimal, placeholder_type_node.clone())
                    } else {
                        Literal::UInt(0, Radix::Decimal, placeholder_type_node.clone())
                    }
                }
                Some(lit) => match lit {
                    Literal::UInt(val, _, _) => {
                        Literal::UInt(val + 1, Radix::Decimal, placeholder_type_node.clone())
                    }
                    Literal::Int(val, _, _) => {
                        Literal::Int(val + 1, Radix::Decimal, placeholder_type_node.clone())
                    }
                    lit => bail!("Invalid enum discriminant literal: {lit:?}"),
                },
            },
            Some(lit) => lit.clone().map_node(context)?,
        };
        match &discr {
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
        variant_discrs.push(discr);
    }

    // Finally, we can figure out the discriminant type
    let discr_type: TypeNode = match discr_type {
        Some(ty) => ty.clone().map_node(context)?,
        None => {
            if min_value >= i8::MIN as i64 && max_value <= i8::MAX as u64 {
                Type::Int8.map_node(context)?
            } else if min_value >= i16::MIN as i64 && max_value <= i16::MAX as u64 {
                Type::Int16.map_node(context)?
            } else if min_value >= i32::MIN as i64 && max_value <= i32::MAX as u64 {
                Type::Int32.map_node(context)?
            } else if max_value <= i64::MAX as u64 {
                // Note: no need to check `min_value` since that's always in the `i64` bounds.
                Type::Int64.map_node(context)?
            } else {
                bail!("Enum repr not set and magnitude exceeds i64::MAX");
            }
        }
    };
    for lit in variant_discrs.iter_mut() {
        match lit {
            Literal::UInt(_, _, type_node) | Literal::Int(_, _, type_node) => {
                *type_node = discr_type.clone();
            }
            _ => unreachable!(),
        }
    }

    Ok(DiscriminantInfo {
        discr_type,
        variant_discrs,
    })
}
