/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn protocol(cbi: &general::CallbackInterface, context: &Context) -> Result<Protocol> {
    Ok(Protocol {
        // Use the main name for the protocol, the callback interface class will get the `Impl`
        // suffix.
        name: cbi.name.clone(),
        base_classes: vec!["typing.Protocol".to_string()],
        methods: cbi.methods.clone().map_node(context)?,
        docstring: cbi.docstring.clone(),
    })
}

pub fn callback_interface_name(cbi: &general::CallbackInterface) -> String {
    names::type_name(&format!("{}Impl", cbi.name))
}
