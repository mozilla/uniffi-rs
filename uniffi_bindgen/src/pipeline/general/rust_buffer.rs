/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! RustBuffer-related FFI functions

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    namespace.ffi_rustbuffer_alloc =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_alloc", &namespace.crate_name));
    namespace.ffi_rustbuffer_from_bytes = RustFfiFunctionName(format!(
        "ffi_{}_rustbuffer_from_bytes",
        &namespace.crate_name
    ));
    namespace.ffi_rustbuffer_free =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_free", &namespace.crate_name));
    namespace.ffi_rustbuffer_reserve =
        RustFfiFunctionName(format!("ffi_{}_rustbuffer_reserve", &namespace.crate_name));
    namespace.ffi_definitions.extend([
        FfiFunction {
            name: RustFfiFunctionName(format!("ffi_{}_rustbuffer_alloc", &namespace.crate_name)),
            async_data: None,
            arguments: vec![FfiArgument {
                name: "size".to_string(),
                ty: FfiType::UInt64.into(),
            }],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None).into()),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferAlloc,
            ..FfiFunction::default()
        }
        .into(),
        FfiFunction {
            name: RustFfiFunctionName(format!(
                "ffi_{}_rustbuffer_from_bytes",
                &namespace.crate_name
            )),
            async_data: None,
            arguments: vec![FfiArgument {
                name: "bytes".to_string(),
                ty: FfiType::ForeignBytes.into(),
            }],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None).into()),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFromBytes,
            ..FfiFunction::default()
        }
        .into(),
        FfiFunction {
            name: RustFfiFunctionName(format!("ffi_{}_rustbuffer_free", &namespace.crate_name)),
            async_data: None,
            arguments: vec![FfiArgument {
                name: "buf".to_string(),
                ty: FfiType::RustBuffer(None).into(),
            }],
            return_type: FfiReturnType { ty: None },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferFree,
            ..FfiFunction::default()
        }
        .into(),
        FfiFunction {
            name: RustFfiFunctionName(format!("ffi_{}_rustbuffer_reserve", &namespace.crate_name)),
            async_data: None,
            arguments: vec![
                FfiArgument {
                    name: "buf".to_string(),
                    ty: FfiType::RustBuffer(None).into(),
                },
                FfiArgument {
                    name: "additional".to_string(),
                    ty: FfiType::UInt64.into(),
                },
            ],
            return_type: FfiReturnType {
                ty: Some(FfiType::RustBuffer(None).into()),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::RustBufferReserve,
            ..FfiFunction::default()
        }
        .into(),
    ]);
    Ok(())
}
