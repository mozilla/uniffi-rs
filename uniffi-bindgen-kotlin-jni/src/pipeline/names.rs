/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use heck::ToUpperCamelCase;

pub fn escape_rust(name: &str) -> String {
    format!("r#{name}")
}

pub fn class_name_kt(name: &str, is_used_as_error: bool) -> String {
    let mut name = name.to_upper_camel_case();
    if is_used_as_error {
        if let Some(start) = name.strip_suffix("Error") {
            name = format!("{start}Exception")
        }
    }
    format!("`{name}`")
}
