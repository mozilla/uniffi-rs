/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn ffi_clone_name(interface_name: &str, context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(uniffi_meta::clone_fn_symbol_name(
        &context.crate_name()?,
        interface_name,
    )))
}

pub fn ffi_free_name(interface_name: &str, context: &Context) -> Result<RustFfiFunctionName> {
    Ok(RustFfiFunctionName(uniffi_meta::free_fn_symbol_name(
        &context.crate_name()?,
        interface_name,
    )))
}

pub fn ffi_definitions(
    namespace: &initial::Namespace,
    context: &Context,
) -> Result<Vec<FfiDefinition>> {
    let namespace_name = &namespace.name;
    let mut ffi_defs = vec![];
    namespace.try_visit(|int: &initial::Interface| {
        ffi_defs.push(FfiDefinition::RustFunction(FfiFunction {
            name: ffi_clone_name(&int.name, context)?,
            async_data: None,
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
                }),
            }],
            return_type: FfiReturnType {
                ty: Some(FfiType::Handle(if int.imp.has_struct() {
                    HandleKind::StructInterface {
                        namespace: namespace_name.clone(),
                        interface_name: int.name.to_string(),
                    }
                } else {
                    HandleKind::TraitInterface {
                        namespace: namespace_name.clone(),
                        interface_name: int.name.to_string(),
                    }
                })),
            },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::ObjectClone,
        }));
        ffi_defs.push(FfiDefinition::RustFunction(FfiFunction {
            name: ffi_free_name(&int.name, context)?,
            async_data: None,
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
                }),
            }],
            return_type: FfiReturnType { ty: None },
            has_rust_call_status_arg: true,
            kind: FfiFunctionKind::ObjectFree,
        }));
        Ok(())
    })?;
    Ok(ffi_defs)
}

pub fn constructors(
    constructors: Vec<initial::Constructor>,
    context: &Context,
) -> Result<Vec<Constructor>> {
    constructors
        .into_iter()
        .filter_map(
            |cons| match exclude::should_exclude_method(&cons.name, context) {
                Err(e) => Some(Err(e)),
                Ok(true) => None,
                Ok(false) => Some(cons.map_node(context)),
            },
        )
        .collect()
}

pub fn methods(methods: Vec<initial::Method>, context: &Context) -> Result<Vec<Method>> {
    methods
        .into_iter()
        .filter_map(
            |meth| match exclude::should_exclude_method(&meth.name, context) {
                Err(e) => Some(Err(e)),
                Ok(true) => None,
                Ok(false) => Some(meth.map_node(context)),
            },
        )
        .collect()
}
