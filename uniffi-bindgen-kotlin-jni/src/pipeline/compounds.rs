/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_optional(input: general::OptionalType, context: &Context) -> Result<OptionalType> {
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    layout_builder.extend(&Type::Boolean, context)?;
    let some_offset = layout_builder.extend(&input.inner.ty, context)?;

    Ok(OptionalType {
        inner: input.inner.map_node(context)?,
        self_type: input.self_type.map_node(context)?,
        some_offset,
    })
}

pub fn map_sequence(input: general::SequenceType, context: &Context) -> Result<SequenceType> {
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    layout_builder.extend(&input.inner.ty, context)?;
    layout_builder.pad_to_align();

    let is_primitive_array = matches!(
        input.inner.ty,
        Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::Int64
            | Type::UInt64
            | Type::Float32
            | Type::Float64
    );

    Ok(SequenceType {
        inner: input.inner.map_node(context)?,
        self_type: input.self_type.map_node(context)?,
        item_size: layout_builder.size(),
        is_primitive_array,
    })
}

pub fn map_map(input: general::MapType, context: &Context) -> Result<MapType> {
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    layout_builder.extend(&input.key.ty, context)?;
    let value_offset = layout_builder.extend(&input.value.ty, context)?;
    layout_builder.pad_to_align();

    Ok(MapType {
        self_type: input.self_type.map_node(context)?,
        key: input.key.map_node(context)?,
        value: input.value.map_node(context)?,
        value_offset,
        item_size: layout_builder.size(),
    })
}

pub fn map_set(input: general::SetType, context: &Context) -> Result<SetType> {
    let mut layout_builder = FfiBufferLayoutBuilder::new();
    layout_builder.extend(&input.inner.ty, context)?;
    layout_builder.pad_to_align();

    Ok(SetType {
        inner: input.inner.map_node(context)?,
        self_type: input.self_type.map_node(context)?,
        item_size: layout_builder.size(),
    })
}

impl BoxedType {
    pub fn jni_into_inner_name(&self) -> String {
        format!("boxIntoInner{}", self.self_type.id)
    }

    pub fn jni_from_ffi_values_name(&self) -> String {
        format!("boxFromFfiValues{}", self.self_type.id)
    }
}
