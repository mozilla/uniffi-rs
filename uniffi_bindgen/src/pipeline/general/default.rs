/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(namespace: &mut Namespace) -> Result<()> {
    // fields and arguments need `DefaultValue` conversion.
    namespace.visit_mut(|arg: &mut Argument| {
        if let Some(DefaultValue::Default(ref mut type_node)) = arg.default {
            *type_node = arg.ty.clone();
        }
    });
    namespace.visit_mut(|field: &mut Field| {
        if let Some(DefaultValue::Default(ref mut type_node)) = field.default {
            *type_node = field.ty.clone();
        }
    });
    Ok(())
}
