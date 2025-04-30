/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Initial IR, this is the Metadata from uniffi_meta with some slight changes:
//!
//! * The Type/Literal enums are wrapped in TypeNode/LiteralNode structs. This allows for future pipeline passes to add fields.
//! * module_path is normalized to module_name (UDL and proc-macros determine module_path differently).

#[macro_use]
pub mod nodes;

mod callable;
mod callback_interfaces;
mod checksums;
mod enums;
mod ffi_async_data;
mod ffi_functions;
mod ffi_types;
mod objects;
mod rust_buffer;
mod rust_future;
mod self_types;
mod sort;
mod type_definitions_from_api;
mod type_nodes;

use crate::pipeline::initial;
use anyhow::{bail, Result};
use indexmap::IndexMap;
pub use nodes::*;
use uniffi_pipeline::{new_pipeline, Node, Pipeline};

/// General IR pipeline
///
/// This is the shared beginning for all bindings pipelines.
/// Bindings generators will add language-specific passes to this.
pub fn pipeline() -> Pipeline<initial::Root, Root> {
    new_pipeline()
        .convert_ir_pass::<Root>()
        .pass(callable::pass)
        .pass(rust_buffer::pass)
        .pass(rust_future::pass)
        .pass(self_types::pass)
        .pass(type_definitions_from_api::pass)
        .pass(type_nodes::pass)
        .pass(ffi_types::pass)
        .pass(ffi_async_data::pass)
        .pass(enums::pass)
        .pass(objects::pass)
        .pass(callback_interfaces::pass)
        .pass(ffi_functions::pass)
        .pass(checksums::pass)
        .pass(sort::pass)
}
