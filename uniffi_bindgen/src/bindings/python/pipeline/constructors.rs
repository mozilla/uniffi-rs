/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use heck::ToSnakeCase;

use super::interface_base_classes as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    fn mutate_constructor(cons: &mut Constructor) -> Result<()> {
        let primary = match &mut cons.callable.kind {
            CallableKind::Constructor { primary, .. } => primary,
            _ => bail!("Invalid callable kind for constructor: {:?}", cons.callable.kind),
        };
        // Python constructors can't be async.  If the primary constructor from Rust is async, then treat
        // it like a secondary constructor which generates a factory method.
        if cons.callable.is_async {
            *primary = false
        }
        if *primary {
            cons.callable.name = "__init__".to_string();
        } else {
            cons.callable.name = cons.callable.name.to_snake_case();
        }
        Ok(())
    }
}
