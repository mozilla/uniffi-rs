/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn name(callable: &general::Callable) -> String {
    if callable.is_primary_constructor() {
        "__init__".to_string()
    } else {
        names::function_name(&callable.name)
    }
}
