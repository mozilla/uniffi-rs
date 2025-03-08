/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::interface_protocols as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    struct Interface {
        +base_classes: Vec<String>,
    }

    fn add_interface_base_classes(int: &prev::Interface) -> Vec<String> {
        std::iter::once(int.protocol.name.clone())
            .chain(int.trait_impls.iter().map(|t| t.trait_name.clone()))
            .chain(int.self_type
                .is_used_as_error
                .then(|| "Exception".to_string())
            )
            .collect()
    }
}
