/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add FFI scaffolding function info

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    let crate_name = namespace.crate_name.clone();
    let mut ffi_definitions = vec![];

    namespace.visit_mut(|callable: &mut Callable| {
        let name = &callable.name;
        let ffi_func_name = match &callable.kind {
            CallableKind::Function => uniffi_meta::fn_symbol_name(&crate_name, name),
            CallableKind::Method { self_type } => {
                uniffi_meta::method_symbol_name(&crate_name, self_type.ty.name().unwrap(), name)
            }
            CallableKind::Constructor { interface_name, .. } => {
                uniffi_meta::constructor_symbol_name(&crate_name, interface_name, name)
            }
            // VTable methods for callback interfaces don't have FFI functions for them
            CallableKind::VTableMethod { .. } => return,
        };
        callable.ffi_func = RustFfiFunctionName(ffi_func_name.clone());

        let receiver_argument = match &callable.kind {
            CallableKind::Method { self_type } => Some(FfiArgument {
                name: "uniffi_self".to_string(),
                ty: self_type.ffi_type.clone(),
            }),
            _ => None,
        };
        let base_arguments = callable.arguments.iter().map(|arg| FfiArgument {
            name: arg.name.clone(),
            ty: arg.ty.ffi_type.clone(),
        });

        ffi_definitions.push(if callable.async_data.is_none() {
            FfiFunction {
                name: RustFfiFunctionName(ffi_func_name),
                async_data: None,
                arguments: receiver_argument
                    .into_iter()
                    .chain(base_arguments)
                    .collect(),
                return_type: FfiReturnType {
                    ty: callable
                        .return_type
                        .ty
                        .as_ref()
                        .map(|ty| ty.ffi_type.clone()),
                },
                has_rust_call_status_arg: true,
                kind: FfiFunctionKind::Scaffolding,
                ..FfiFunction::default()
            }
            .into()
        } else {
            FfiFunction {
                name: RustFfiFunctionName(ffi_func_name),
                async_data: callable.async_data.clone(),
                arguments: receiver_argument
                    .into_iter()
                    .chain(base_arguments)
                    .collect(),
                return_type: FfiReturnType {
                    ty: Some(FfiType::Handle(HandleKind::RustFuture).into()),
                },
                has_rust_call_status_arg: false,
                kind: FfiFunctionKind::Scaffolding,
                ..FfiFunction::default()
            }
            .into()
        });
    });

    namespace.ffi_definitions.extend(ffi_definitions);
    Ok(())
}
