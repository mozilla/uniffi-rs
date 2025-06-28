/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add [TypeNode::ffi_type]

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let module_name = module.name.clone();
    module.visit_mut(|node: &mut TypeNode| {
        node.ffi_type = FfiTypeNode::from(generate_ffi_type(&node.ty, &module_name));
    });
    Ok(())
}

fn generate_ffi_type(ty: &Type, current_module_name: &str) -> FfiType {
    match ty {
        // Types that are the same map to themselves, naturally.
        Type::UInt8 => FfiType::UInt8,
        Type::Int8 => FfiType::Int8,
        Type::UInt16 => FfiType::UInt16,
        Type::Int16 => FfiType::Int16,
        Type::UInt32 => FfiType::UInt32,
        Type::Int32 => FfiType::Int32,
        Type::UInt64 => FfiType::UInt64,
        Type::Int64 => FfiType::Int64,
        Type::Float32 => FfiType::Float32,
        Type::Float64 => FfiType::Float64,
        // Booleans lower into an Int8, to work around a bug in JNA.
        Type::Boolean => FfiType::Int8,
        // Strings are always owned rust values.
        // We might add a separate type for borrowed strings in future.
        Type::String => FfiType::RustBuffer(None),
        // Byte strings are also always owned rust values.
        // We might add a separate type for borrowed byte strings in future as well.
        Type::Bytes => FfiType::RustBuffer(None),
        // Objects are pointers to an Arc<>
        Type::Interface {
            module_name, name, ..
        } => FfiType::Handle(HandleKind::Interface {
            module_name: module_name.clone(),
            interface_name: name.clone(),
        }),
        // Callback interfaces are passed as opaque integer handles.
        Type::CallbackInterface { module_name, name } => FfiType::Handle(HandleKind::Interface {
            module_name: module_name.clone(),
            interface_name: name.clone(),
        }),
        // Other types are serialized into a bytebuffer and deserialized on the other side.
        Type::Enum { module_name, .. } | Type::Record { module_name, .. } => {
            FfiType::RustBuffer((module_name != current_module_name).then_some(module_name.clone()))
        }
        Type::Optional { .. }
        | Type::Sequence { .. }
        | Type::Map { .. }
        | Type::Timestamp
        | Type::Duration => FfiType::RustBuffer(None),
        Type::Custom {
            module_name,
            builtin,
            ..
        } => {
            match generate_ffi_type(builtin, current_module_name) {
                // Fixup `module_name` for primitive types that lower to `RustBuffer`.
                //
                // This is needed to handle external custom types, where the builtin type is
                // something like `String`.
                FfiType::RustBuffer(None) if module_name != current_module_name => {
                    FfiType::RustBuffer(Some(module_name.clone()))
                }
                ffi_type => ffi_type,
            }
        }
    }
}
