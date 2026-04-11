/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_callback_interface(
    input: general::CallbackInterface,
    context: &Context,
) -> Result<CallbackInterface> {
    Ok(CallbackInterface {
        self_type: input.self_type.map_node(context)?,
        methods: map_methods(
            &input.name,
            input.vtable.methods.into_iter().map(|m| m.callable),
            context,
        )?,
        name: input.name,
        orig_name: input.orig_name,
        module_path: input.module_path,
        docstring: input.docstring,
        crate_name: context.current_crate_name()?.to_string(),
        for_trait_interface: false,
    })
}

pub fn map_trait_interface(
    input: general::Interface,
    context: &Context,
) -> Result<CallbackInterface> {
    Ok(CallbackInterface {
        self_type: input.self_type.map_node(context)?,
        methods: map_methods(
            &input.name,
            input.methods.into_iter().map(|m| m.callable),
            context,
        )?,
        name: input.name,
        orig_name: input.orig_name,
        module_path: input.module_path,
        docstring: input.docstring,
        crate_name: context.current_crate_name()?.to_string(),
        for_trait_interface: true,
    })
}

pub fn interface_for_callback_interface(
    cbi: &general::CallbackInterface,
    context: &Context,
) -> Result<Interface> {
    Ok(Interface {
        name: cbi.name.to_upper_camel_case(),
        methods: cbi.methods.clone().map_node(context)?,
        docstring: cbi.docstring.clone(),
    })
}

fn map_methods(
    interface_name: &str,
    methods: impl Iterator<Item = general::Callable>,
    context: &Context,
) -> Result<Vec<CallbackMethod>> {
    methods
        .map(|callable| {
            Ok(CallbackMethod {
                dispatch_fn_rs: format!(
                    "uniffi_callback_dispatch_{}_{}_{}",
                    context.namespace_name()?.to_snake_case(),
                    interface_name.to_snake_case(),
                    callable.name.to_snake_case(),
                ),
                dispatch_fn_kt: format!(
                    "callbackInterface{}{}{}",
                    context.namespace_name()?.to_upper_camel_case(),
                    interface_name.to_upper_camel_case(),
                    callable.name.to_upper_camel_case(),
                ),
                callable: callable.map_node(context)?,
            })
        })
        .collect()
}
