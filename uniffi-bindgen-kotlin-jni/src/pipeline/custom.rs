/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_custom(input: general::CustomType, context: &Context) -> Result<CustomType> {
    Ok(CustomType {
        config: context.config()?.custom_types.get(&input.name).cloned(),
        crate_name: context.current_crate_name()?.to_string(),
        self_type: input.self_type.map_node(context)?,
        name: input.name,
        builtin: input.builtin.map_node(context)?,
        docstring: input.docstring,
    })
}

impl CustomType {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_upper_camel_case())
    }
}
