/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use uniffi_meta;

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let crate_name = module.crate_name.clone();
    let module_name = module.name.clone();
    let mut ffi_definitions = vec![];
    module.visit_mut(|int: &mut Interface| {
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
                    ty: FfiType::Handle(HandleKind::Interface {
                        module_name: module_name.clone(),
                        interface_name: int.name.to_string(),
                    })
                    .into(),
                }],
                return_type: FfiReturnType {
                    ty: Some(
                        FfiType::Handle(HandleKind::Interface {
                            module_name: module_name.clone(),
                            interface_name: int.name.to_string(),
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
                    ty: FfiType::Handle(HandleKind::Interface {
                        module_name: module_name.clone(),
                        interface_name: int.name.to_string(),
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
        UniffiTraitMethods::fill_from(&mut int.uniffi_trait_methods, &int.uniffi_traits);
    });
    module.ffi_definitions.extend(ffi_definitions);
    Ok(())
}
