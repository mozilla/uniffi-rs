/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Put this first so we can use the macros
#[macro_use]
mod nodes;

mod callable;
mod callback_interfaces;
mod checksums;
mod ffi_functions;
mod ffi_types;
mod objects;
mod organize;
mod rust_buffer;
mod rust_future;
mod self_types;
mod sort;
mod type_definitions_from_api;
mod type_nodes;

use crate::pipeline::{general, initial};
use anyhow::{bail, Result};
use indexmap::IndexMap;
use uniffi_pipeline::{new_pipeline, Node, Pipeline};

use nodes::*;

/// General IR pipeline
///
/// This is the shared beginning for all bindings pipelines.
/// Bindings generators will add language-specific passes to this.
pub fn general_pipeline() -> Pipeline<initial::Root, general::Root> {
    new_pipeline()
        .convert_ir_pass::<Root>()
        .pass(organize::pass)
        .pass(callable::pass)
        .pass(self_types::pass)
        .pass(type_definitions_from_api::pass)
        .pass(type_nodes::pass)
        .pass(ffi_types::pass)
        .pass(ffi_functions::pass)
        .pass(rust_buffer::pass)
        .pass(rust_future::pass)
        .pass(objects::pass)
        .pass(callback_interfaces::pass)
        .pass(checksums::pass)
        .pass(sort::pass)
        .convert_ir_pass::<general::Root>()
}
