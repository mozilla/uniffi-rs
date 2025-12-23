/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta;

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    let crate_name = namespace.crate_name.clone();
    let namespace_name = namespace.name.clone();
    let mut ffi_definitions = vec![];
    namespace.visit_mut(|int: &mut Interface| {
        int.ffi_func_clone =
            RustFfiFunctionName(uniffi_meta::clone_fn_symbol_name(&crate_name, &int.name));
        int.ffi_func_free =
            RustFfiFunctionName(uniffi_meta::free_fn_symbol_name(&crate_name, &int.name));
        ffi_definitions.push(
            FfiFunction {
                name: int.ffi_func_clone.clone(),
                is_async: false,
                arguments: vec![FfiArgument {
                    name: "ptr".to_string(),
                    ty: FfiType::Handle(if int.imp.has_struct() {
                        HandleKind::StructInterface {
                            namespace: namespace_name.clone(),
                            interface_name: int.name.to_string(),
                        }
                    } else {
                        HandleKind::TraitInterface {
                            namespace: namespace_name.clone(),
                            interface_name: int.name.to_string(),
                        }
                    })
                    .into(),
                }],
                return_type: FfiReturnType {
                    ty: Some(
                        FfiType::Handle(if int.imp.has_struct() {
                            HandleKind::StructInterface {
                                namespace: namespace_name.clone(),
                                interface_name: int.name.to_string(),
                            }
                        } else {
                            HandleKind::TraitInterface {
                                namespace: namespace_name.clone(),
                                interface_name: int.name.to_string(),
                            }
                        })
                        .into(),
                    ),
                },
                has_rust_call_status_arg: true,
                kind: FfiFunctionKind::ObjectClone,
                ..FfiFunction::default()
            }
            .into(),
        );
        ffi_definitions.push(
            FfiFunction {
                name: int.ffi_func_free.clone(),
                is_async: false,
                arguments: vec![FfiArgument {
                    name: "ptr".to_string(),
                    ty: FfiType::Handle(if int.imp.has_struct() {
                        HandleKind::StructInterface {
                            namespace: namespace_name.clone(),
                            interface_name: int.name.to_string(),
                        }
                    } else {
                        HandleKind::TraitInterface {
                            namespace: namespace_name.clone(),
                            interface_name: int.name.to_string(),
                        }
                    })
                    .into(),
                }],
                return_type: FfiReturnType { ty: None },
                has_rust_call_status_arg: true,
                kind: FfiFunctionKind::ObjectFree,
                ..FfiFunction::default()
            }
            .into(),
        );
    });
    namespace.ffi_definitions.extend(ffi_definitions);
    Ok(())
}
