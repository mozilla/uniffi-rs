/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use heck::ToUpperCamelCase;

pub fn function_async_data(
    func: &initial::Function,
    context: &Context,
) -> Result<Option<AsyncData>> {
    if !func.is_async {
        return Ok(None);
    }
    let ffi_return_type = func
        .return_type
        .as_ref()
        .map(|ty| ffi_types::ffi_type(ty, context))
        .transpose()?;
    async_data(context, ffi_return_type.as_ref()).map(Some)
}

pub fn method_async_data(meth: &initial::Method, context: &Context) -> Result<Option<AsyncData>> {
    if !meth.is_async {
        return Ok(None);
    }
    let ffi_return_type = meth
        .return_type
        .as_ref()
        .map(|ty| ffi_types::ffi_type(ty, context))
        .transpose()?;
    async_data(context, ffi_return_type.as_ref()).map(Some)
}

pub fn constructor_async_data(
    cons: &initial::Constructor,
    interface_name: &str,
    imp: &ObjectImpl,
    context: &Context,
) -> Result<Option<AsyncData>> {
    let namespace = context.namespace_name()?;
    if !cons.is_async {
        return Ok(None);
    }
    let ffi_return_type = ffi_types::interface_ffi_type(&namespace, interface_name, imp)?;
    async_data(context, Some(&ffi_return_type)).map(Some)
}

fn async_data(context: &Context, ffi_return_type: Option<&FfiType>) -> Result<AsyncData> {
    let crate_name = context.crate_name()?;
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
        Some(FfiType::Handle(_)) => "u64",
        Some(FfiType::RustBuffer(_)) => "rust_buffer",
        None => "void",
        ty => panic!("Invalid future return type: {ty:?}"),
    };
    let struct_crate_name = match ffi_return_type {
        Some(FfiType::RustBuffer(Some(rust_buffer_crate))) => rust_buffer_crate,
        _ => "",
    };

    Ok(AsyncData {
        ffi_rust_future_poll: RustFfiFunctionName(format!(
            "ffi_{crate_name}_rust_future_poll_{return_type_name}"
        )),
        ffi_rust_future_cancel: RustFfiFunctionName(format!(
            "ffi_{crate_name}_rust_future_cancel_{return_type_name}"
        )),
        ffi_rust_future_complete: RustFfiFunctionName(format!(
            "ffi_{crate_name}_rust_future_complete_{return_type_name}"
        )),
        ffi_rust_future_free: RustFfiFunctionName(format!(
            "ffi_{crate_name}_rust_future_free_{return_type_name}"
        )),
        ffi_foreign_future_result: FfiStructName(format!(
            "ForeignFutureResult{struct_crate_name}{}",
            return_type_name.to_upper_camel_case()
        )),
        ffi_foreign_future_complete: FfiFunctionTypeName(format!(
            "ForeignFutureComplete{struct_crate_name}{return_type_name}"
        )),
    })
}
