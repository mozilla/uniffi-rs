/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_root(input: initial::Root, context: &Context) -> Result<Root> {
    let mut context = context.clone();
    context.update_from_root(&input)?;

    Ok(Root {
        cdylib: input.cdylib,
        namespaces: input.namespaces.map_node(&context)?,
        builtin_types: builtin_types(&context)?,
    })
}

fn builtin_types(context: &Context) -> Result<BuiltinTypes> {
    Ok(BuiltinTypes {
        i8: Type::Int8.map_node(context)?,
        u8: Type::UInt8.map_node(context)?,
        i16: Type::Int16.map_node(context)?,
        u16: Type::UInt16.map_node(context)?,
        i32: Type::Int32.map_node(context)?,
        u32: Type::UInt32.map_node(context)?,
        i64: Type::Int64.map_node(context)?,
        u64: Type::UInt64.map_node(context)?,
        f32: Type::Float32.map_node(context)?,
        f64: Type::Float64.map_node(context)?,
        string: Type::String.map_node(context)?,
    })
}
