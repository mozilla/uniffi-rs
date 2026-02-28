/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::{Expr, Lit, Meta};

pub fn extract_docstring(current: &mut Option<String>, meta: &Meta) {
    let Ok(name_value) = meta.require_name_value() else {
        return;
    };
    if name_value.path.is_ident("doc") {
        if let Expr::Lit(expr) = &name_value.value {
            if let Lit::Str(lit_str) = &expr.lit {
                let lit_value = lit_str.value();
                let docstring = lit_value.trim();
                match current.as_mut() {
                    None => *current = Some(docstring.to_owned()),
                    Some(current) => {
                        current.push('\n');
                        current.push_str(docstring);
                    }
                }
            }
        }
    }
}
