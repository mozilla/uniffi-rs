/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_variants(
    variants: Vec<general::Variant>,
    shape: EnumShape,
    context: &Context,
) -> Result<Vec<Variant>> {
    variants
        .into_iter()
        .map(|v| {
            Ok(Variant {
                name: if shape.is_error() {
                    names::type_name(&v.name)
                } else {
                    names::non_error_variant_name(&v.name)
                },
                discr: v.discr.map_node(context)?,
                fields_kind: v.fields_kind,
                fields: v.fields.map_node(context)?,
                docstring: v.docstring,
            })
        })
        .collect()
}

pub fn enum_variant_name(name: &str, ty: &general::TypeNode) -> Result<String> {
    Ok(match &ty.ty {
        Type::Enum { .. } => {
            // Assume enum literals are not error types
            names::non_error_variant_name(name)
        }
        type_kind => {
            bail!("Invalid type for enum literal: {type_kind:?}")
        }
    })
}
