/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI info for callback interfaces

use super::*;
use std::collections::HashSet;

pub fn pass(module: &mut Module) -> Result<()> {
    let crate_name = module.crate_name.clone();
    module.try_visit_mut(|cbi: &mut CallbackInterface| {
        cbi.vtable = vtable(&crate_name, &cbi.name, cbi.methods.clone())?;
        Ok(())
    })?;
    module.try_visit_mut(|int: &mut Interface| {
        if int.imp.has_callback_interface() {
            int.vtable = Some(vtable(&crate_name, &int.name, int.methods.clone())?);
        }
        Ok(())
    })?;
    // Convert CallableKind:Method to CallableKind:VTableMethod
    module.try_visit_mut(|cbi: &mut CallbackInterface| {
        for meth in cbi.methods.iter_mut() {
            meth.callable.kind = match meth.callable.kind.take() {
                CallableKind::Method { interface_name } => CallableKind::VTableMethod {
                    trait_name: interface_name,
                },
                kind => bail!("Unexpected callable kind: {kind:?}"),
            };
        }
        Ok(())
    })?;
    module.try_visit_mut(|vtable: &mut VTable| {
        for meth in vtable.methods.iter_mut() {
            meth.callable.kind = match meth.callable.kind.take() {
                CallableKind::Method { interface_name } => CallableKind::VTableMethod {
                    trait_name: interface_name,
                },
                kind => bail!("Unexpected callable kind: {kind:?}"),
            };
        }
        Ok(())
    })?;

    add_vtable_ffi_definitions(module)?;
    Ok(())
}

fn vtable(crate_name: &str, interface_name: &str, methods: Vec<Method>) -> Result<VTable> {
    Ok(VTable {
        struct_type: FfiType::Struct(FfiStructName(format!(
            "VTableCallbackInterface{}",
            interface_name
        ))),
        interface_name: interface_name.to_string(),
        init_fn: RustFfiFunctionName(uniffi_meta::init_callback_vtable_fn_symbol_name(
            crate_name,
            interface_name,
        )),
        methods: methods
            .into_iter()
            .enumerate()
            .map(|(i, meth)| {
                Ok(VTableMethod {
                    callable: meth.callable,
                    ffi_type: FfiType::Function(FfiFunctionTypeName(format!(
                        "CallbackInterface{}Method{i}",
                        interface_name
                    ))),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn add_vtable_ffi_definitions(module: &mut Module) -> Result<()> {
    let mut ffi_definitions = vec![];
    let mut seen_return_types = HashSet::new();
    let module_name = module.name.clone();
    module.try_visit(|vtable: &VTable| {
        let interface_name = &vtable.interface_name;
        // FFI Function Type for each method in the VTable
        for (i, meth) in vtable.methods.iter().enumerate() {
            let method_name = format!("CallbackInterface{interface_name}Method{i}");
            match &meth.callable.async_data {
                Some(async_data) => {
                    ffi_definitions.push(vtable_method_async(
                        &module_name,
                        interface_name,
                        method_name,
                        async_data,
                        &meth.callable,
                    )?);
                }
                None => {
                    ffi_definitions.push(vtable_method(
                        &module_name,
                        interface_name,
                        method_name,
                        &meth.callable,
                    ));
                }
            }
            // Async-related FFI definitions
            let Some(async_info) = &meth.callable.async_data else {
                continue;
            };
            let ffi_return_type = meth
                .callable
                .return_type
                .ty
                .clone()
                .map(|return_type| return_type.ffi_type);

            if seen_return_types.insert(ffi_return_type.clone()) {
                ffi_definitions.extend([
                    FfiStruct {
                        name: async_info.ffi_foreign_future_result.clone(),
                        fields: match ffi_return_type {
                            Some(return_ffi_type) => vec![
                                FfiField::new("return_value", return_ffi_type),
                                FfiField::new("call_status", FfiType::RustCallStatus),
                            ],
                            None => vec![
                                // In Rust, `return_value` is `()` -- a ZST.
                                // ZSTs are not valid in `C`, but they also take up 0 space.
                                // Skip the `return_value` field to make the layout correct.
                                FfiField::new("call_status", FfiType::RustCallStatus),
                            ],
                        },
                    }
                    .into(),
                    FfiFunctionType {
                        name: async_info.ffi_foreign_future_complete.clone(),
                        arguments: vec![
                            FfiArgument::new(
                                "callback_data",
                                FfiType::Handle(HandleKind::ForeignFuture),
                            ),
                            FfiArgument::new(
                                "result",
                                FfiType::Struct(async_info.ffi_foreign_future_result.clone()),
                            ),
                        ],
                        return_type: FfiReturnType { ty: None },
                        has_rust_call_status_arg: false,
                    }
                    .into(),
                ]);
            }
        }
        // FFIStruct for the VTable
        ffi_definitions.extend([
            FfiFunctionType {
                name: FfiFunctionTypeName(format!(
                    "CallbackInterfaceFree{module_name}_{interface_name}"
                )),
                arguments: vec![FfiArgument::new(
                    "handle",
                    FfiType::Handle(HandleKind::CallbackInterface {
                        module_name: module_name.to_string(),
                        interface_name: interface_name.to_string(),
                    }),
                )],
                return_type: FfiReturnType { ty: None },
                has_rust_call_status_arg: false,
            }
            .into(),
            FfiStruct {
                name: FfiStructName(format!("VTableCallbackInterface{}", interface_name)),
                fields: vtable
                    .methods
                    .iter()
                    .map(|vtable_meth| {
                        FfiField::new(&vtable_meth.callable.name, vtable_meth.ffi_type.clone())
                    })
                    .chain([FfiField::new(
                        "uniffi_free",
                        FfiType::Function(FfiFunctionTypeName(format!(
                            "CallbackInterfaceFree{module_name}_{interface_name}"
                        ))),
                    )])
                    .collect(),
            }
            .into(),
        ]);
        // FFI Function to initialize the VTable
        ffi_definitions.push(
            FfiFunction {
                name: vtable.init_fn.clone(),
                arguments: vec![FfiArgument {
                    name: "vtable".into(),
                    ty: FfiType::Reference(Box::new(vtable.struct_type.clone())),
                }],
                return_type: FfiReturnType { ty: None },
                async_data: None,
                has_rust_call_status_arg: false,
                kind: FfiFunctionKind::RustVtableInit,
                ..FfiFunction::default()
            }
            .into(),
        );
        Ok(())
    })?;
    module.ffi_definitions.extend(ffi_definitions);
    Ok(())
}

fn vtable_method(
    module_name: &str,
    interface_name: &str,
    method_name: String,
    callable: &Callable,
) -> FfiDefinition {
    FfiFunctionType {
        name: FfiFunctionTypeName(method_name),
        arguments: std::iter::once(FfiArgument {
            name: "uniffi_handle".into(),
            ty: FfiType::Handle(HandleKind::CallbackInterface {
                module_name: module_name.to_string(),
                interface_name: interface_name.to_string(),
            }),
        })
        .chain(callable.arguments.iter().map(|arg| FfiArgument {
            name: arg.name.clone(),
            ty: arg.ty.ffi_type.clone(),
        }))
        .chain(std::iter::once(match &callable.return_type.ty {
            Some(ty) => FfiArgument {
                name: "uniffi_out_return".into(),
                ty: FfiType::MutReference(Box::new(ty.ffi_type.clone())),
            },
            None => FfiArgument {
                name: "uniffi_out_return".into(),
                ty: FfiType::VoidPointer,
            },
        }))
        .collect(),
        has_rust_call_status_arg: true,
        return_type: FfiReturnType { ty: None },
    }
    .into()
}

fn vtable_method_async(
    module_name: &str,
    interface_name: &str,
    method_name: String,
    async_data: &AsyncData,
    callable: &Callable,
) -> Result<FfiDefinition> {
    Ok(FfiFunctionType {
        name: FfiFunctionTypeName(method_name),
        arguments: std::iter::once(FfiArgument {
            name: "uniffi_handle".into(),
            ty: FfiType::Handle(HandleKind::CallbackInterface {
                module_name: module_name.to_string(),
                interface_name: interface_name.to_string(),
            }),
        })
        .chain(callable.arguments.iter().map(|arg| FfiArgument {
            name: arg.name.clone(),
            ty: arg.ty.ffi_type.clone(),
        }))
        .chain([
            FfiArgument {
                name: "uniffi_future_callback".into(),
                ty: FfiType::Function(async_data.ffi_foreign_future_complete.clone()),
            },
            FfiArgument {
                name: "uniffi_callback_data".into(),
                ty: FfiType::Handle(HandleKind::ForeignFutureCallbackData),
            },
            FfiArgument {
                name: "uniffi_out_return".into(),
                ty: FfiType::MutReference(Box::new(FfiType::Struct(FfiStructName(
                    "ForeignFuture".to_owned(),
                )))),
            },
        ])
        .collect(),
        has_rust_call_status_arg: false,
        return_type: FfiReturnType { ty: None },
    }
    .into())
}
