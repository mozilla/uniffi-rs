/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Define a protocol for each interface

use super::docstrings as prev;
use crate::nanopass::ir;

ir! {
    extend prev;

    struct Interface {
        +protocol: Protocol
    }

    /// Protocol implemented by an interface
    +struct Protocol {
        name: String,
        /// Base class for the protocol.  Most of the type this is `typing.Protocol`.
        base_class: String,
        docstring: Option<String>,
        methods: Vec<Method>,
    }

    fn add_interface_protocol(int: &mut prev::Interface, context: &Context) -> Result<Protocol> {
        if int.imp.has_callback_interface() {
            // This is a trait interface that can be implemented in Python, so it is treated like a
            // callback interface where the primary use-case is the trait being implemented
            // locally.  It is a base-class local implementations might subclass.
            // We reuse "Protocol.py" for this, even though here we are not generating a protocol

            // In this case, users will mostly be interacting with the protocol so give it the main name.
            let protocol_name = int.name.clone();
            int.name = format!("{}Impl", int.name);

            Ok(Protocol {
                name: protocol_name,
                base_class: "".to_string(),
                docstring: int.docstring.clone(),
                methods: int.methods.clone().into_ir(context)?,
            })
        } else {
            // Regular interface.  In this case, users will mostly be interacting with the
            // interface, so give it the main naame.
            Ok(Protocol {
                name: format!("{}Protocol", int.name),
                base_class: "typing.Protocol".to_string(),
                docstring: int.docstring.clone(),
                methods: int.methods.clone().into_ir(context)?,
            })
        }
    }
}
