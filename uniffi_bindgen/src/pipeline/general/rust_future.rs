/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn ffi_definitions(namespace: &initial::Namespace) -> Result<Vec<FfiDefinition>> {
    if !has_async_fns(namespace) {
        return Ok(vec![]);
    }

    let crate_name = namespace.crate_name.clone();
    let mut ffi_defs = vec![];

    ffi_defs.extend([FfiFunctionType {
        name: FfiFunctionTypeName("RustFutureContinuationCallback".to_owned()),
        arguments: vec![
            FfiArgument::new("data", FfiType::Handle(HandleKind::RustFuture)),
            FfiArgument::new("poll_result", FfiType::Int8),
        ],
        return_type: FfiReturnType { ty: None },
        has_rust_call_status_arg: false,
    }
    .into()]);

    let all_async_return_types = [
        (Some(FfiType::UInt8), "u8"),
        (Some(FfiType::Int8), "i8"),
        (Some(FfiType::UInt16), "u16"),
        (Some(FfiType::Int16), "i16"),
        (Some(FfiType::UInt32), "u32"),
        (Some(FfiType::Int32), "i32"),
        (Some(FfiType::UInt64), "u64"),
        (Some(FfiType::Int64), "i64"),
        (Some(FfiType::Float32), "f32"),
        (Some(FfiType::Float64), "f64"),
        (
            Some(FfiType::Handle(HandleKind::StructInterface {
                namespace: "".into(),
                interface_name: "".into(),
            })),
            "u64",
        ),
        (
            Some(FfiType::Handle(HandleKind::TraitInterface {
                namespace: "".into(),
                interface_name: "".into(),
            })),
            "u64",
        ),
        (Some(FfiType::RustBuffer(None)), "rust_buffer"),
        (None, "void"),
    ];
    for (return_type, return_type_name) in all_async_return_types {
        let poll_name = format!("ffi_{crate_name}_rust_future_poll_{return_type_name}");
        ffi_defs.push(ffi_rust_future_poll(poll_name));

        let cancel_name = format!("ffi_{crate_name}_rust_future_cancel_{return_type_name}");
        ffi_defs.push(ffi_rust_future_cancel(cancel_name));

        let complete_name = format!("ffi_{crate_name}_rust_future_complete_{return_type_name}");
        ffi_defs.push(ffi_rust_future_complete(return_type.clone(), complete_name));

        let free_name = format!("ffi_{crate_name}_rust_future_free_{return_type_name}");
        ffi_defs.push(ffi_rust_future_free(free_name));
    }
    Ok(ffi_defs)
}

fn ffi_rust_future_poll(symbol_name: String) -> FfiDefinition {
    FfiFunction {
        name: RustFfiFunctionName(symbol_name),
        async_data: None,
        arguments: vec![
            FfiArgument {
                name: "handle".to_owned(),
                ty: FfiType::Handle(HandleKind::RustFuture),
            },
            FfiArgument {
                name: "callback".to_owned(),
                ty: FfiType::Function(FfiFunctionTypeName(
                    "RustFutureContinuationCallback".to_owned(),
                )),
            },
            FfiArgument {
                name: "callback_data".to_owned(),
                ty: FfiType::Handle(HandleKind::RustFuture),
            },
        ],
        return_type: FfiReturnType { ty: None },
        has_rust_call_status_arg: false,
        kind: FfiFunctionKind::RustFuturePoll,
    }
    .into()
}

fn ffi_rust_future_cancel(symbol_name: String) -> FfiDefinition {
    FfiFunction {
        name: RustFfiFunctionName(symbol_name),
        async_data: None,
        arguments: vec![FfiArgument {
            name: "handle".to_owned(),
            ty: FfiType::Handle(HandleKind::RustFuture),
        }],
        return_type: FfiReturnType { ty: None },
        has_rust_call_status_arg: false,
        kind: FfiFunctionKind::RustFutureCancel,
    }
    .into()
}

fn ffi_rust_future_complete(return_type: Option<FfiType>, symbol_name: String) -> FfiDefinition {
    FfiFunction {
        name: RustFfiFunctionName(symbol_name),
        async_data: None,
        arguments: vec![FfiArgument {
            name: "handle".to_owned(),
            ty: FfiType::Handle(HandleKind::RustFuture),
        }],
        return_type: FfiReturnType { ty: return_type },
        has_rust_call_status_arg: true,
        kind: FfiFunctionKind::RustFutureComplete,
    }
    .into()
}

fn ffi_rust_future_free(symbol_name: String) -> FfiDefinition {
    FfiFunction {
        name: RustFfiFunctionName(symbol_name),
        async_data: None,
        arguments: vec![FfiArgument {
            name: "handle".to_owned(),
            ty: FfiType::Handle(HandleKind::RustFuture),
        }],
        return_type: FfiReturnType { ty: None },
        has_rust_call_status_arg: false,
        kind: FfiFunctionKind::RustFutureFree,
    }
    .into()
}

fn has_async_fns(namespace: &initial::Namespace) -> bool {
    namespace.has_descendant(|func: &initial::Function| func.is_async)
        || namespace.has_descendant(|meth: &initial::Method| meth.is_async)
        || namespace.has_descendant(|cons: &initial::Constructor| cons.is_async)
}
