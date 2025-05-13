/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn pass(cbi: &mut CallbackInterface) -> Result<()> {
    cbi.protocol = Protocol {
        // Use the main name for the protocol, the callback interface class will get the `Impl`
        // suffix.
        name: cbi.name.clone(),
        base_classes: vec!["typing.Protocol".to_string()],
        methods: cbi.methods.clone(),
        docstring: cbi.docstring.clone(),
    };
    cbi.name = format!("{}Impl", cbi.name);
    Ok(())
}
