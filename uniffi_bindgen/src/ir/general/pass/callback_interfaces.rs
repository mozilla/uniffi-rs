/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! FFI info for callback interfaces

use super::*;

pub fn step(module: &mut Module) -> Result<()> {
    let crate_name = module.crate_name.clone();
    let mut ffi_definitions = vec![];

    module.try_visit_mut(|cbi: &mut CallbackInterface| {
        cbi.vtable = vtable(&crate_name, &cbi.name, cbi.methods.take())?;
        add_vtable_ffi_definitions(&mut ffi_definitions, &cbi.name, &cbi.vtable)?;
        Ok(())
    })?;
    module.try_visit_mut(|int: &mut Interface| {
        if int.imp.has_callback_interface() {
            let vtable = vtable(&crate_name, &int.name, int.methods.take())?;
            add_vtable_ffi_definitions(&mut ffi_definitions, &int.name, &vtable)?;
            int.vtable = Some(vtable);
        }
        Ok(())
    })?;
    module.ffi_definitions.extend(ffi_definitions);
    Ok(())
}

fn vtable(crate_name: &str, interface_name: &str, methods: Vec<Method>) -> Result<VTable> {
    Ok(VTable {
        struct_type: FfiType::Struct(format!("VTableCallbackInterface{}", interface_name)),
        init_fn: RustFfiFunctionName(uniffi_meta::init_callback_vtable_fn_symbol_name(
            crate_name,
            interface_name,
        )),
        methods: methods
            .into_iter()
            .enumerate()
            .map(|(i, mut meth)| {
                meth.callable.kind = match meth.callable.kind {
                    CallableKind::Method { interface_name } => CallableKind::VTableMethod {
                        trait_name: interface_name,
                    },
                    kind => bail!("Unexpected callable kind: {kind:?}"),
                };
                Ok(VTableMethod {
                    callable: meth.callable,
                    ffi_type: FfiType::Function(format!(
                        "CallbackInterface{}Method{i}",
                        interface_name
                    )),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn add_vtable_ffi_definitions(
    ffi_definitions: &mut Vec<FfiDefinition>,
    interface_name: &str,
    vtable: &VTable,
) -> Result<()> {
    // FFI Function to initialize the VTable
    ffi_definitions.push(
        FfiFunction {
            name: vtable.init_fn.0.clone(),
            arguments: vec![FfiArgument {
                name: "vtable".into(),
                ty: FfiType::Reference(Box::new(vtable.struct_type.clone())),
            }],
            return_type: FfiReturnType { ty: None },
            is_async: false,
            has_rust_call_status_arg: false,
            kind: FfiFunctionKind::RustVtableInit,
        }
        .into(),
    );

    // FFI Function Type for each method in the VTable
    for (i, meth) in vtable.methods.iter().enumerate() {
        let method_name = format!("CallbackInterface{interface_name}Method{i}");
        match &meth.callable.async_data {
            Some(async_data) => {
                ffi_definitions.push(vtable_method_async(
                    async_data,
                    method_name,
                    &meth.callable,
                )?);
            }
            None => {
                ffi_definitions.push(vtable_method(method_name, &meth.callable));
            }
        }
    }
    Ok(())
}

fn vtable_method(method_name: String, callable: &Callable) -> FfiDefinition {
    FfiFunctionType {
        name: method_name,
        arguments: std::iter::once(FfiArgument {
            name: "uniffi_handle".into(),
            ty: FfiType::UInt64,
        })
        .chain(callable.arguments.iter().map(|arg| {
            FfiArgument! {
                name: arg.name.clone(),
                ty: arg.ty.ffi_type.clone(),
            }
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
    async_data: &AsyncData,
    method_name: String,
    callable: &Callable,
) -> Result<FfiDefinition> {
    Ok(FfiFunctionType {
        name: method_name,
        arguments: std::iter::once(FfiArgument {
            name: "uniffi_handle".into(),
            ty: FfiType::UInt64,
        })
        .chain(callable.arguments.iter().map(|arg| {
            FfiArgument! {
                name: arg.name.clone(),
                ty: arg.ty.ffi_type.clone(),
            }
        }))
        .chain([
            FfiArgument {
                name: "uniffi_future_callback".into(),
                ty: async_data.ffi_rust_future_complete.clone(),
            },
            FfiArgument {
                name: "uniffi_callback_data".into(),
                ty: FfiType::UInt64,
            },
            FfiArgument {
                name: "uniffi_out_return".into(),
                ty: FfiType::MutReference(Box::new(FfiType::Struct("ForeignFuture".to_owned()))),
            },
        ])
        .collect(),
        has_rust_call_status_arg: false,
        return_type: FfiReturnType { ty: None },
    }
    .into())
}
