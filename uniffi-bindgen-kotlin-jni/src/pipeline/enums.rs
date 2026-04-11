/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_enum(en: general::Enum, context: &Context) -> Result<Enum> {
    let mut context = context.clone();
    context.update_from_enum(&en);
    let discr_type = en.discr_type.map_node(&context)?;

    let kotlin_kind = if matches!(en.shape, EnumShape::Error { flat: true }) {
        KotlinEnumKind::FlatError
    } else if en.self_type.is_used_as_error || !en.is_flat {
        KotlinEnumKind::SealedClass
    } else {
        KotlinEnumKind::EnumClass {
            discr_type: en.discr_specified.then(|| discr_type.type_kt.clone()),
        }
    };

    Ok(Enum {
        is_flat: en.is_flat,
        use_entries: context.config()?.use_enum_entries(),
        self_type: en.self_type.map_node(&context)?,
        discr_type,
        discr_specified: en.discr_specified,
        variants: en.variants.map_node(&context)?,
        name: en.name,
        orig_name: en.orig_name,
        shape: en.shape,
        kotlin_kind,
        docstring: en.docstring,
        recursive: en.recursive,
    })
}

pub fn map_variant(variant: general::Variant, context: &Context) -> Result<Variant> {
    let en = context.current_enum()?;
    let name_kt = if !en.is_flat || matches!(en.shape, EnumShape::Error { flat: true }) {
        names::class_name_kt(&variant.name, en.self_type.is_used_as_error)
    } else {
        format!("`{}`", variant.name.to_shouty_snake_case())
    };
    let discr: LiteralNode = variant.discr.map_node(context)?;

    Ok(Variant {
        name_kt,
        name: variant.name,
        orig_name: variant.orig_name,
        discr,
        fields_kind: variant.fields_kind,
        fields: records::map_fields(variant.fields, context)?,
        docstring: variant.docstring,
    })
}
