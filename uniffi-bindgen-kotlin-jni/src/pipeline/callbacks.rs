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
        methods: map_methods(&input.name, input.vtable, context)?,
        name: input.name,
        module_path: input.module_path,
        docstring: input.docstring,
        crate_name: context.current_crate_name()?.to_string(),
    })
}

fn map_methods(
    interface_name: &str,
    vtable: general::VTable,
    context: &Context,
) -> Result<Vec<CallbackMethod>> {
    vtable
        .methods
        .into_iter()
        .map(|m| {
            Ok(CallbackMethod {
                dispatch_fn_rs: format!(
                    "uniffi_callback_dispatch_{}_{}_{}",
                    context.namespace_name()?.to_snake_case(),
                    interface_name.to_snake_case(),
                    m.callable.name.to_snake_case(),
                ),
                dispatch_fn_kt: format!(
                    "callbackInterface{}{}{}",
                    context.namespace_name()?.to_upper_camel_case(),
                    interface_name.to_upper_camel_case(),
                    m.callable.name.to_upper_camel_case(),
                ),
                callable: m.callable.map_node(context)?,
            })
        })
        .collect()
}
