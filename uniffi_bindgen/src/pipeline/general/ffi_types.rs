/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn ffi_type(ty: &Type, context: &Context) -> Result<FfiType> {
    Ok(match ty {
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
            namespace,
            name,
            imp,
            ..
        } => interface_ffi_type(namespace, name, imp)?,
        // Callback interfaces are passed as opaque integer handles.
        Type::CallbackInterface { namespace, name } => {
            FfiType::Handle(HandleKind::TraitInterface {
                namespace: namespace.clone(),
                interface_name: name.clone(),
            })
        }
        // Other types are serialized into a bytebuffer and deserialized on the other side.
        Type::Enum { namespace, .. } | Type::Record { namespace, .. } => FfiType::RustBuffer(
            (*namespace != context.namespace_name()?).then_some(namespace.clone()),
        ),
        Type::Optional { .. }
        | Type::Sequence { .. }
        | Type::Map { .. }
        | Type::Timestamp
        | Type::Duration => FfiType::RustBuffer(None),
        Type::Custom {
            namespace, builtin, ..
        } => {
            match ffi_type(builtin, context)? {
                // Fixup `module_name` for primitive types that lower to `RustBuffer`.
                //
                // This is needed to handle external custom types, where the builtin type is
                // something like `String`.
                FfiType::RustBuffer(None) if *namespace != context.namespace_name()? => {
                    FfiType::RustBuffer(Some(namespace.clone()))
                }
                ffi_type => ffi_type,
            }
        }
    })
}

pub fn interface_ffi_type(
    namespace: &str,
    interface_name: &str,
    imp: &ObjectImpl,
) -> Result<FfiType> {
    let kind = if imp.has_struct() {
        HandleKind::StructInterface {
            namespace: namespace.to_string(),
            interface_name: interface_name.to_string(),
        }
    } else {
        HandleKind::TraitInterface {
            namespace: namespace.to_string(),
            interface_name: interface_name.to_string(),
        }
    };
    Ok(FfiType::Handle(kind))
}
