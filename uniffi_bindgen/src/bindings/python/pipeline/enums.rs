/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use heck::ToUpperCamelCase;

pub fn map_enum(en: general::Enum, context: &Context) -> Result<Enum> {
    Ok(Enum {
        name: names::type_name(&en.name),
        is_flat: en.is_flat,
        variants: map_variants(&en.name, en.variants, en.shape, context)?,
        shape: en.shape.map_node(context)?,
        discr_type: en.discr_type.map_node(context)?,
        docstring: en.docstring,
        self_type: en.self_type.map_node(context)?,
        constructors: interfaces::map_constructors(&en.name, en.constructors, context)?,
        methods: en.methods.map_node(context)?,
        uniffi_trait_methods: en.uniffi_trait_methods.map_node(context)?,
        recursive: en.recursive,
    })
}

pub fn map_variants(
    enum_name: &str,
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
                class_name_py: format!(
                    "UniffiEnumVariant{}{}",
                    enum_name.to_upper_camel_case(),
                    v.name.to_upper_camel_case(),
                ),
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
