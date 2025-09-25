/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(cb: &mut Callable) -> Result<()> {
    // normalize the custom type to its builtin type
    if let Some(node) = cb.throws_type.ty.as_mut() {
        if let Type::Custom { builtin, .. } = &node.ty {
            node.ty = *builtin.clone();
        }
    }
    Ok(())
}
