/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_callable(input: general::Callable, context: &Context) -> Result<Callable> {
    Ok(Callable {
        kind: input.kind.map_node(context)?,
        name: input.name,
        arguments: input.arguments.map_node(context)?,
        return_type: input.return_type.ty.map_node(context)?,
    })
}

pub fn function_jni_method_name(func: &general::Function, context: &Context) -> Result<String> {
    Ok(format!(
        "function{}{}",
        context.current_crate_name()?.to_upper_camel_case(),
        func.callable.name.to_upper_camel_case()
    ))
}
