/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let module_name = module.name.clone();
    module.visit_mut(|node: &mut FfiTypeNode| {
        node.type_name = ffi_type_name(&module_name, &node.ty);
    });
    module.try_visit_mut(|meth: &mut VTableMethod| {
        meth.ffi_default_value = match &meth.callable.return_type.ty {
            Some(type_node) => match &type_node.ffi_type.ty {
                FfiType::UInt8
                | FfiType::Int8
                | FfiType::UInt16
                | FfiType::Int16
                | FfiType::UInt32
                | FfiType::Int32
                | FfiType::UInt64
                | FfiType::Int64
                | FfiType::Handle(_) => "0".to_string(),
                FfiType::Float32 | FfiType::Float64 => "0.0".to_string(),
                FfiType::RustBuffer(Some(buf_module_name)) if *buf_module_name != module_name => {
                    format!("{buf_module_name}._UniffiRustBuffer.default()")
                }
                FfiType::RustBuffer(_) => "_UniffiRustBuffer.default()".to_string(),
                ffi_type => bail!("Invalid VTable return type: {ffi_type:?}"),
            },
            // When we need to use a value for void returns, we use a `u8` placeholder and `0` as
            // the default.
            None => "0".to_string(),
        };
        Ok(())
    })?;
    Ok(())
}

fn ffi_type_name(module_name: &str, ffi_type: &FfiType) -> String {
    match ffi_type {
        FfiType::Int8 => "ctypes.c_int8".to_string(),
        FfiType::UInt8 => "ctypes.c_uint8".to_string(),
        FfiType::Int16 => "ctypes.c_int16".to_string(),
        FfiType::UInt16 => "ctypes.c_uint16".to_string(),
        FfiType::Int32 => "ctypes.c_int32".to_string(),
        FfiType::UInt32 => "ctypes.c_uint32".to_string(),
        FfiType::Int64 => "ctypes.c_int64".to_string(),
        FfiType::UInt64 => "ctypes.c_uint64".to_string(),
        FfiType::Float32 => "ctypes.c_float".to_string(),
        FfiType::Float64 => "ctypes.c_double".to_string(),
        FfiType::Handle(_) => "ctypes.c_uint64".to_string(),
        FfiType::RustBuffer(Some(buf_module_name)) if buf_module_name != module_name => {
            format!("{buf_module_name}._UniffiRustBuffer")
        }
        FfiType::RustBuffer(_) => "_UniffiRustBuffer".to_string(),
        FfiType::RustCallStatus => "_UniffiRustCallStatus".to_string(),
        FfiType::ForeignBytes => "_UniffiForeignBytes".to_string(),
        FfiType::Function(name) => name.0.clone(),
        FfiType::Struct(name) => name.0.clone(),
        // Pointer to an `asyncio.EventLoop` instance
        FfiType::Reference(inner) | FfiType::MutReference(inner) => {
            format!("ctypes.POINTER({})", ffi_type_name(module_name, inner))
        }
        FfiType::VoidPointer => "ctypes.c_void_p".to_string(),
    }
}
