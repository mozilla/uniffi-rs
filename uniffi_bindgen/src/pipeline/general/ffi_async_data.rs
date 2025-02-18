/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let namespace = module.crate_name.clone();

    // Change `is_async` to `async_data`
    module.visit_mut(|callable: &mut Callable| {
        callable.async_data = callable
            .is_async
            .then(|| generate_async_data(&namespace, callable.ffi_return_type()));
    });
    module.visit_mut(|ffi_func: &mut FfiFunction| {
        ffi_func.async_data = ffi_func
            .is_async
            .then(|| generate_async_data(&namespace, ffi_func.return_type.ty.as_ref()));
    });
    Ok(())
}

fn generate_async_data(namespace: &str, ffi_return_type: Option<&FfiType>) -> AsyncData {
    let return_type_name = match ffi_return_type {
        Some(FfiType::UInt8) => "u8",
        Some(FfiType::Int8) => "i8",
        Some(FfiType::UInt16) => "u16",
        Some(FfiType::Int16) => "i16",
        Some(FfiType::UInt32) => "u32",
        Some(FfiType::Int32) => "i32",
        Some(FfiType::UInt64) => "u64",
        Some(FfiType::Int64) => "i64",
        Some(FfiType::Float32) => "f32",
        Some(FfiType::Float64) => "f64",
        Some(FfiType::RustArcPtr { .. }) => "pointer",
        Some(FfiType::RustBuffer(_)) => "rust_buffer",
        None => "void",
        ty => panic!("Invalid future return type: {ty:?}"),
    };
    AsyncData {
        ffi_rust_future_poll: RustFfiFunctionName(format!(
            "ffi_{namespace}_rust_future_poll_{return_type_name}"
        )),
        ffi_rust_future_cancel: RustFfiFunctionName(format!(
            "ffi_{namespace}_rust_future_cancel_{return_type_name}"
        )),
        ffi_rust_future_complete: RustFfiFunctionName(format!(
            "ffi_{namespace}_rust_future_complete_{return_type_name}"
        )),
        ffi_rust_future_free: RustFfiFunctionName(format!(
            "ffi_{namespace}_rust_future_free_{return_type_name}"
        )),
        ffi_foreign_future_result: FfiStructName(format!("ForeignFutureResult{return_type_name}")),
        ffi_foreign_future_complete: FfiFunctionTypeName(format!(
            "ForeignFutureComplete{return_type_name}"
        )),
    }
}
