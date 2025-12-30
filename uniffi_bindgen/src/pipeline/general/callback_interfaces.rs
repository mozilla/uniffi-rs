/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI info for callback interfaces

use super::*;
use heck::ToUpperCamelCase;

pub fn vtable(methods: &[initial::Method], context: &Context) -> Result<VTable> {
    let crate_name = context.crate_name()?;
    let namespace_name = context.namespace_name()?;
    let interface_name = context.current_type_name()?;

    Ok(VTable {
        struct_type: vtable_struct_type(&namespace_name, &interface_name),
        interface_name: interface_name.to_string(),
        init_fn: RustFfiFunctionName(uniffi_meta::init_callback_vtable_fn_symbol_name(
            &crate_name,
            &interface_name,
        )),
        clone_fn_type: FfiFunctionTypeName(format!(
            "CallbackInterfaceClone{}_{interface_name}",
            namespace_name.to_upper_camel_case(),
        )),
        free_fn_type: FfiFunctionTypeName(format!(
            "CallbackInterfaceFree{}_{interface_name}",
            namespace_name.to_upper_camel_case(),
        )),
        methods: methods
            .iter()
            .enumerate()
            .map(|(i, meth)| {
                let meth: Method = meth.clone().map_node(context)?;
                Ok(VTableMethod {
                    callable: meth.callable,
                    ffi_type: FfiType::Function(FfiFunctionTypeName(format!(
                        "CallbackInterface{}{}Method{i}",
                        namespace_name.to_upper_camel_case(),
                        interface_name
                    ))),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

pub fn vtable_for_interface(int: &initial::Interface, context: &Context) -> Result<Option<VTable>> {
    Ok(match int.imp {
        ObjectImpl::CallbackTrait => Some(vtable(&int.methods, context)?),
        _ => None,
    })
}

pub fn ffi_definitions(
    namespace: &initial::Namespace,
    context: &Context,
) -> Result<Vec<FfiDefinition>> {
    let namespace_name = &namespace.name;
    let crate_name = &namespace.crate_name;
    let mut ffi_defs = vec![];
    let mut has_async_method = false;
    namespace.try_visit(|int: &initial::Interface| {
        if int.imp.has_callback_interface() {
            if int.methods.iter().any(|m| m.is_async) {
                has_async_method = true;
            }
            add_vtable_ffi_defs(
                crate_name,
                namespace_name,
                &int.name,
                &int.methods,
                context,
                &mut ffi_defs,
            )?;
        }
        Ok(())
    })?;
    namespace.try_visit(|cbi: &initial::CallbackInterface| {
        if cbi.methods.iter().any(|m| m.is_async) {
            has_async_method = true;
        }
        add_vtable_ffi_defs(
            crate_name,
            namespace_name,
            &cbi.name,
            &cbi.methods,
            context,
            &mut ffi_defs,
        )?;
        Ok(())
    })?;
    if has_async_method {
        ffi_defs.extend([
            FfiFunctionType {
                name: FfiFunctionTypeName("ForeignFutureDroppedCallback".to_owned()),
                arguments: vec![FfiArgument::new(
                    "handle",
                    FfiType::Handle(HandleKind::ForeignFuture),
                )],
                return_type: FfiReturnType { ty: None },
                has_rust_call_status_arg: false,
            }
            .into(),
            FfiStruct {
                name: FfiStructName("ForeignFutureDroppedCallbackStruct".to_owned()),
                fields: vec![
                    FfiField::new("handle", FfiType::Handle(HandleKind::ForeignFuture)),
                    FfiField::new(
                        "free",
                        FfiType::Function(FfiFunctionTypeName(
                            "ForeignFutureDroppedCallback".to_owned(),
                        )),
                    ),
                ],
            }
            .into(),
        ]);
    }

    Ok(ffi_defs)
}

fn add_vtable_ffi_defs(
    crate_name: &str,
    namespace_name: &str,
    interface_name: &str,
    methods: &[initial::Method],
    context: &Context,
    ffi_defs: &mut Vec<FfiDefinition>,
) -> Result<()> {
    // Method definitions
    for (i, meth) in methods.iter().enumerate() {
        let method_name = format!(
            "CallbackInterface{}{}Method{i}",
            namespace_name.to_upper_camel_case(),
            interface_name,
        );
        ffi_defs.extend(vtable_method_ffi_defs(
            namespace_name,
            interface_name,
            &method_name,
            meth,
            context,
        )?);
    }
    let handle_type = FfiType::Handle(HandleKind::TraitInterface {
        namespace: namespace_name.to_string(),
        interface_name: interface_name.to_string(),
    });
    // Free/clone method definitions
    ffi_defs.extend([
        FfiFunctionType {
            name: FfiFunctionTypeName(format!(
                "CallbackInterfaceClone{}_{interface_name}",
                namespace_name.to_upper_camel_case(),
            )),
            arguments: vec![FfiArgument::new("handle", handle_type.clone())],
            return_type: FfiReturnType {
                ty: Some(handle_type.clone()),
            },
            has_rust_call_status_arg: false,
        }
        .into(),
        FfiFunctionType {
            name: FfiFunctionTypeName(format!(
                "CallbackInterfaceFree{}_{interface_name}",
                namespace_name.to_upper_camel_case(),
            )),
            arguments: vec![FfiArgument::new("handle", handle_type.clone())],
            return_type: FfiReturnType { ty: None },
            has_rust_call_status_arg: false,
        }
        .into(),
    ]);
    // VTable struct definition
    ffi_defs.push(vtable_struct(namespace_name, interface_name, methods)?);
    // Init function
    ffi_defs.push(
        FfiFunction {
            name: vtable_init_fn(crate_name, interface_name),
            arguments: vec![FfiArgument {
                name: "vtable".into(),
                ty: FfiType::Reference(Box::new(vtable_struct_type(
                    namespace_name,
                    interface_name,
                ))),
            }],
            return_type: FfiReturnType { ty: None },
            async_data: None,
            has_rust_call_status_arg: false,
            kind: FfiFunctionKind::RustVtableInit,
        }
        .into(),
    );
    Ok(())
}

fn vtable_struct(
    namespace_name: &str,
    interface_name: &str,
    methods: &[initial::Method],
) -> Result<FfiDefinition> {
    let mut fields = vec![
        FfiField::new(
            "uniffi_free",
            FfiType::Function(FfiFunctionTypeName(format!(
                "CallbackInterfaceFree{}_{interface_name}",
                namespace_name.to_upper_camel_case(),
            ))),
        ),
        FfiField::new(
            "uniffi_clone",
            FfiType::Function(FfiFunctionTypeName(format!(
                "CallbackInterfaceClone{}_{interface_name}",
                namespace_name.to_upper_camel_case(),
            ))),
        ),
    ];
    for (i, meth) in methods.iter().enumerate() {
        let function_type = format!(
            "CallbackInterface{}{}Method{i}",
            namespace_name.to_upper_camel_case(),
            interface_name,
        );
        fields.push(FfiField {
            name: meth.name.clone(),
            ty: FfiType::Function(FfiFunctionTypeName(function_type)),
        });
    }

    Ok(FfiStruct {
        name: FfiStructName(format!(
            "VTableCallbackInterface{}{}",
            namespace_name.to_upper_camel_case(),
            interface_name
        )),
        fields,
    }
    .into())
}

fn vtable_method_ffi_defs(
    namespace_name: &str,
    interface_name: &str,
    method_name: &str,
    meth: &initial::Method,
    context: &Context,
) -> Result<impl Iterator<Item = FfiDefinition>> {
    let mut vtable_arguments = vec![FfiArgument {
        name: "uniffi_handle".into(),
        ty: FfiType::Handle(HandleKind::TraitInterface {
            namespace: namespace_name.to_string(),
            interface_name: interface_name.to_string(),
        }),
    }];
    for arg in meth.inputs.iter() {
        vtable_arguments.push(FfiArgument {
            name: arg.name.clone(),
            ty: ffi_types::ffi_type(&arg.ty, context)?,
        });
    }
    let ffi_return_type = match &meth.return_type {
        Some(ty) => Some(ffi_types::ffi_type(ty, context)?),
        None => None,
    };
    let async_data = ffi_async_data::method_async_data(meth, context)?;
    match &async_data {
        None => {
            vtable_arguments.push(match &ffi_return_type {
                Some(ffi_type) => FfiArgument {
                    name: "uniffi_out_return".into(),
                    ty: FfiType::MutReference(Box::new(ffi_type.clone())),
                },
                None => FfiArgument {
                    name: "uniffi_out_return".into(),
                    ty: FfiType::VoidPointer,
                },
            });
        }
        Some(async_data) => {
            vtable_arguments.extend([
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
                        "ForeignFutureDroppedCallbackStruct".to_owned(),
                    )))),
                },
            ]);
        }
    }

    let defs = [FfiFunctionType {
        name: FfiFunctionTypeName(method_name.to_string()),
        arguments: vtable_arguments,
        has_rust_call_status_arg: async_data.is_none(),
        return_type: FfiReturnType { ty: None },
    }
    .into()]
    .into_iter()
    .chain(
        async_data
            .map(|async_data| {
                [
                    FfiStruct {
                        name: async_data.ffi_foreign_future_result.clone(),
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
                        name: async_data.ffi_foreign_future_complete.clone(),
                        arguments: vec![
                            FfiArgument::new(
                                "callback_data",
                                FfiType::Handle(HandleKind::ForeignFuture),
                            ),
                            FfiArgument::new(
                                "result",
                                FfiType::Struct(async_data.ffi_foreign_future_result.clone()),
                            ),
                        ],
                        return_type: FfiReturnType { ty: None },
                        has_rust_call_status_arg: false,
                    }
                    .into(),
                ]
            })
            .into_iter()
            .flatten(),
    );

    Ok(defs)
}

fn vtable_struct_type(namespace_name: &str, interface_name: &str) -> FfiType {
    FfiType::Struct(FfiStructName(format!(
        "VTableCallbackInterface{}{}",
        namespace_name.to_upper_camel_case(),
        interface_name
    )))
}

fn vtable_init_fn(crate_name: &str, interface_name: &str) -> RustFfiFunctionName {
    RustFfiFunctionName(uniffi_meta::init_callback_vtable_fn_symbol_name(
        crate_name,
        interface_name,
    ))
}
