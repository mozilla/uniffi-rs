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

    Ok(SequenceType {
        inner: input.inner.map_node(context)?,
        self_type: input.self_type.map_node(context)?,
        item_size: layout_builder.size(),
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
