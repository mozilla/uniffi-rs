/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::type_names as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    /// Map FfiTypes to the Python type names
    fn map_ffi_type(ffi_type: prev::FfiType) -> String {
        match ffi_type {
            prev::FfiType::Int8 => "ctypes.c_int8".to_string(),
            prev::FfiType::UInt8 => "ctypes.c_uint8".to_string(),
            prev::FfiType::Int16 => "ctypes.c_int16".to_string(),
            prev::FfiType::UInt16 => "ctypes.c_uint16".to_string(),
            prev::FfiType::Int32 => "ctypes.c_int32".to_string(),
            prev::FfiType::UInt32 => "ctypes.c_uint32".to_string(),
            prev::FfiType::Int64 => "ctypes.c_int64".to_string(),
            prev::FfiType::UInt64 => "ctypes.c_uint64".to_string(),
            prev::FfiType::Float32 => "ctypes.c_float".to_string(),
            prev::FfiType::Float64 => "ctypes.c_double".to_string(),
            prev::FfiType::Handle => "ctypes.c_uint64".to_string(),
            prev::FfiType::RustArcPtr => "ctypes.c_void_p".to_string(),
            prev::FfiType::RustBuffer(module_name) => match module_name {
                None => "_UniffiRustBuffer".to_string(),
                Some(module_name) => format!("{module_name}._UniffiRustBuffer"),
            },
            prev::FfiType::RustCallStatus => "_UniffiRustCallStatus".to_string(),
            prev::FfiType::ForeignBytes => "_UniffiForeignBytes".to_string(),
            prev::FfiType::Function(name) => name,
            prev::FfiType::Struct(name) => name,
            prev::FfiType::Reference(inner) | prev::FfiType::MutReference(inner) => {
                format!("ctypes.POINTER({})", map_ffi_type(*inner))
            }
            prev::FfiType::VoidPointer => "ctypes.c_void_p".to_string(),
        }
    }

    impl Callable {
        // This returns a string now
        fn ffi_return_type(&self) -> Option<&str> {
            self.return_type.ty.as_ref().map(|ty| ty.ffi_type.as_str())
        }
    }
}
