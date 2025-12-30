/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! RustBuffer-related FFI functions

use super::*;

pub fn rustbuffer_alloc_fn_name(context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(format!(
        "ffi_{}_rustbuffer_alloc",
        context.crate_name()?
    )))
}

pub fn rustbuffer_from_bytes_fn_name(context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(format!(
        "ffi_{}_rustbuffer_from_bytes",
        context.crate_name()?
    )))
}

pub fn rustbuffer_free_fn_name(context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(format!(
        "ffi_{}_rustbuffer_free",
        context.crate_name()?
    )))
}

pub fn rustbuffer_reserve_fn_name(context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(format!(
        "ffi_{}_rustbuffer_reserve",
        context.crate_name()?
    )))
}

pub fn ffi_definitions(context: &Context) -> Result<Vec<FfiDefinition>> {
    Ok([
        FfiFunction {
            name: rustbuffer_alloc_fn_name(context)?,
            async_data: None,
            arguments: vec![FfiArgument {
                name: "size".to_string(),
                ty: FfiType::UInt64,
            }],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferAlloc,
        }
        .into(),
        FfiFunction {
            name: rustbuffer_from_bytes_fn_name(context)?,
            async_data: None,
            arguments: vec![FfiArgument {
                name: "bytes".to_string(),
                ty: FfiType::ForeignBytes,
            }],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFromBytes,
        }
        .into(),
        FfiFunction {
            name: rustbuffer_free_fn_name(context)?,
            async_data: None,
            arguments: vec![FfiArgument {
                name: "buf".to_string(),
                ty: FfiType::RustBuffer(None),
            }],
            return_type: FfiReturnType { ty: None },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFree,
        }
        .into(),
        FfiFunction {
            name: rustbuffer_reserve_fn_name(context)?,
            async_data: None,
            arguments: vec![
                FfiArgument {
                    name: "buf".to_string(),
                    ty: FfiType::RustBuffer(None),
                },
                FfiArgument {
                    name: "additional".to_string(),
                    ty: FfiType::UInt64,
                },
            ],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferReserve,
        }
        .into(),
    ]
    .into())
}
