/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn variant_name_kt(variant: &general::Variant, context: &Context) -> Result<String> {
    Ok(if context.current_enum()?.is_flat {
        format!("`{}`", variant.name.to_shouty_snake_case())
    } else {
        format!("`{}`", variant.name.to_upper_camel_case())
    })
}
