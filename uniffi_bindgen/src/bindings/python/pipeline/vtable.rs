/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::constructors as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    struct VTableMethod {
        /// Placeholder value to use when the method raised an exception and we don't have a return
        /// value
        +default_return_value: String,
    }

    fn add_vtablemethod_default_return_value(vmeth: &prev::VTableMethod) -> String {
        let Some(ffi_type) = vmeth.callable.ffi_return_type() else {
            // When we need to use a value for void returns, we use a `u8` placeholder and `0` as
            // the default.
            return "0".to_string();
        };

        match ffi_type {
            prev::FfiType::UInt8
            | prev::FfiType::Int8
            | prev::FfiType::UInt16
            | prev::FfiType::Int16
            | prev::FfiType::UInt32
            | prev::FfiType::Int32
            | prev::FfiType::UInt64
            | prev::FfiType::Int64
            | prev::FfiType::Handle => "0".to_string(),
            prev::FfiType::Float32 | prev::FfiType::Float64 => "0.0".to_string(),
            prev::FfiType::RustArcPtr => "ctypes.c_void_p()".to_string(),
            prev::FfiType::RustBuffer(module_name) => match module_name {
                None => "_UniffiRustBuffer.default()".to_string(),
                Some(module_name) => {
                    format!("{module_name}._UniffiRustBuffer.default()")
                }
            },
            _ => panic!("Invalid FFI return type for vtable method: {ffi_type:?}"),
        }
    }
}
