/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_record(input: general::Record, context: &Context) -> Result<Record> {
    Ok(Record {
        self_type: input.self_type.map_node(context)?,
        immutable: context.config()?.record_is_immutable(&input.name),
        name: input.name,
        fields_kind: input.fields_kind,
        fields: map_fields(input.fields, context)?,
        docstring: input.docstring,
        recursive: input.recursive,
    })
}

fn map_fields(input: Vec<general::Field>, context: &Context) -> Result<Vec<Field>> {
    let mut ffi_field_counter = 0..;
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    input
        .into_iter()
        .enumerate()
        .map(|(index, input)| {
            let ty = input.ty.map_node(context)?;
            let ffi_fields = ty
                .ffi_types
                .iter()
                .map(|ffi_type| FfiField {
                    index: ffi_field_counter.next().unwrap(),
                    ty: *ffi_type,
                })
                .collect();
            let offset = layout_builder.extend(&ty.ty, context)?;

            Ok(Field {
                name: input.name,
                index,
                ty,
                default: input.default.map_node(context)?,
                docstring: input.docstring,
                ffi_fields,
                offset,
            })
        })
        .collect::<Result<Vec<_>>>()
}

impl Record {
    pub fn name_kt(&self) -> String {
        names::class_name_kt(&self.name, self.self_type.is_used_as_error)
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn ffi_types(&self) -> impl Iterator<Item = &FfiType> {
        self.fields.iter().flat_map(Field::ffi_types)
    }
}

impl Field {
    pub fn name_kt(&self) -> String {
        if self.name.is_empty() {
            format!("v{}", self.index + 1)
        } else {
            format!("`{}`", self.name.to_lower_camel_case())
        }
    }

    pub fn name_rs(&self) -> String {
        if self.name.is_empty() {
            self.index.to_string()
        } else {
            names::escape_rust(&self.name)
        }
    }

    pub fn ffi_types(&self) -> impl Iterator<Item = &FfiType> {
        self.ffi_fields.iter().map(|f| &f.ty)
    }

    pub fn lowers_to_primitive(&self) -> bool {
        self.ffi_fields.len() == 1
    }
}
