/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! RustBuffer-related FFI functions

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    module.ffi_rustbuffer_alloc =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_alloc", &module.crate_name));
    module.ffi_rustbuffer_from_bytes =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_from_bytes", &module.crate_name));
    module.ffi_rustbuffer_free =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_free", &module.crate_name));
    module.ffi_rustbuffer_reserve =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_reserve", &module.crate_name));
    module.ffi_definitions.extend([
        FfiFunction! {
            name: format!("ffi_{}_rustbuffer_alloc", &module.crate_name),
            async_data: None,
            arguments: vec![FfiArgument! {
                name: "size".to_string(),
                ty: FfiType::UInt64,
            }],
            return_type: FfiReturnType! {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferAlloc,
        }
        .into(),
        FfiFunction! {
            name: format!("ffi_{}_rustbuffer_from_bytes", &module.crate_name),
            async_data: None,
            arguments: vec![FfiArgument! {
                name: "bytes".to_string(),
                ty: FfiType::ForeignBytes,
            }],
            return_type: FfiReturnType! {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFromBytes,
        }
        .into(),
        FfiFunction! {
            name: format!("ffi_{}_rustbuffer_free", &module.crate_name),
            async_data: None,
            arguments: vec![FfiArgument! {
                name: "buf".to_string(),
                ty: FfiType::RustBuffer(None),
            }],
            return_type: FfiReturnType! { ty: None },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFree,
        }
        .into(),
        FfiFunction! {
            name: format!("ffi_{}_rustbuffer_reserve", &module.crate_name),
            async_data: None,
            arguments: vec![
                FfiArgument! {
                    name: "buf".to_string(),
                    ty: FfiType::RustBuffer(None),
                },
                FfiArgument! {
                    name: "additional".to_string(),
                    ty: FfiType::UInt64,
                },
            ],
            return_type: FfiReturnType! {
                ty: Some(FfiType::RustBuffer(None)),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferReserve,
        }
        .into(),
    ]);
    Ok(())
}
