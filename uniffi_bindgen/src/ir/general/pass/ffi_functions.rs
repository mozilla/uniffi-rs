/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Add FFI scaffolding function info

use super::*;

pub fn step(root: &mut Root) -> Result<()> {
    root.visit_mut(|module: &mut Module| {
        let crate_name = module.crate_name.clone();
        let mut ffi_definitions = vec![];

        module.visit_mut(|callable: &mut Callable| {
            let name = &callable.name;
            let ffi_func_name = match &callable.kind {
                CallableKind::Function => uniffi_meta::fn_symbol_name(&crate_name, name),
                CallableKind::Method { interface_name } => {
                    uniffi_meta::method_symbol_name(&crate_name, interface_name, name)
                }
                CallableKind::Constructor { interface_name, .. } => {
                    uniffi_meta::constructor_symbol_name(&crate_name, interface_name, name)
                }
                // VTableMethods are introduced later
                kind => unimplemented!("Can't handle callable kind {kind:?}"),
            };
            callable.ffi_func = RustFfiFunctionName(ffi_func_name.clone());
            ffi_definitions.push(
                FfiFunction! {
                    name: ffi_func_name,
                    is_async: callable.is_async(),
                    arguments: callable
                        .arguments
                        .iter()
                        .map(|arg| {
                            FfiArgument! {
                                name: arg.name.clone(),
                                ty: arg.ty.ffi_type.clone(),
                            }
                        })
                        .collect(),
                    return_type: FfiReturnType! {
                        ty: callable
                            .return_type
                            .ty
                            .as_ref()
                            .map(|ty| ty.ffi_type.clone()),
                    },
                    has_rust_call_status_arg: true,
                    is_object_free_function: false,
                }
                .into(),
            );
        });

        module.ffi_definitions.extend(ffi_definitions);
    });
    Ok(())
}
