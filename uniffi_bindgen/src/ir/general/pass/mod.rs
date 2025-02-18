/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Invoke this before the modules, so they can use the macros defined inside.
ir_pass! {
    from: ir::initial;
    to: ir::general;
}

mod callable;
mod callback_interfaces;
mod checksums;
mod ffi_functions;
mod ffi_types;
mod object_handles;
mod organize;
mod rust_buffer;
mod rust_future;
mod self_types;
mod sort;
mod type_definitions_from_api;
mod type_is_used_as_error;

use crate::ir::{self, ir_pass, Pass};
use anyhow::Result;

// Need to manually implement FromNode for enums that get mapped to structs
impl FromNode<ir::initial::Type> for TypeNode {
    fn from_node(ty: ir::initial::Type) -> Result<Self> {
        Ok(TypeNode! {
            ty: ty.into_node()?,
        })
    }
}

pub fn pass() -> Pass<Root> {
    Pass::new("general")
        .step(organize::step)
        .step(callable::step)
        .step(self_types::step)
        .step(type_definitions_from_api::step)
        .step(type_is_used_as_error::step)
        .step(ffi_types::step)
        .step(rust_buffer::step)
        .step(rust_future::step)
        .step(ffi_functions::step)
        .step(object_handles::step)
        .step(callback_interfaces::step)
        .step(checksums::step)
        .step(sort::step)
}
