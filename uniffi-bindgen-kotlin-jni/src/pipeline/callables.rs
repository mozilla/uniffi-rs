/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_function(input: general::Function, context: &Context) -> Result<Function> {
    let module_path = &input.module_path;
    let fully_qualified_name_rs = format!(
        "{module_path}::{}",
        names::escape_rust(&input.callable.orig_name)
    );
    Ok(Function {
        docstring: input.docstring,
        jni_method_name: format!(
            "function{}{}",
            context.current_crate_name()?.to_upper_camel_case(),
            input.callable.name.to_upper_camel_case()
        ),
        callable: map_callable(input.callable, fully_qualified_name_rs, context)?,
    })
}

fn map_callable(
    input: general::Callable,
    fully_qualified_name_rs: String,
    context: &Context,
) -> Result<Callable> {
    Ok(Callable {
        fully_qualified_name_rs,
        kind: input.kind.map_node(context)?,
        is_async: input.async_data.is_some(),
        name: input.name,
    })
}

impl Callable {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }
}
