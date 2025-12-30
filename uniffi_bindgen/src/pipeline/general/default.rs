/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_default_value(
    default: initial::DefaultValue,
    context: &Context,
) -> Result<DefaultValue> {
    Ok(match default {
        initial::DefaultValue::Literal(lit) => DefaultValue::Literal(lit.map_node(context)?),
        initial::DefaultValue::Default => {
            DefaultValue::Default(context.current_arg_or_field_type()?)
        }
    })
}
