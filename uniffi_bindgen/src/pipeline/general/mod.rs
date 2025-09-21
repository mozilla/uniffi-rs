/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! General IR - a useful starting point for language-specific IRs.

#[macro_use]
pub mod nodes;

mod callable;
mod callback_interfaces;
mod checksums;
mod default;
mod enums;
mod ffi_async_data;
mod ffi_functions;
mod ffi_types;
mod modules;
mod objects;
mod records;
mod rename;
mod rust_buffer;
mod rust_future;
mod self_types;
mod sort;
mod type_definitions_from_api;
mod type_nodes;
mod uniffi_traits;

use crate::pipeline::initial;
use anyhow::{bail, Result};
use indexmap::IndexMap;
pub use nodes::*;
use uniffi_pipeline::{new_pipeline, Node, Pipeline};

/// General IR pipeline
///
/// This is the shared beginning for all bindings pipelines.
/// Bindings generators will add language-specific passes to this.
pub fn pipeline(bindings_toml_key: &str) -> Pipeline<initial::Root, Root> {
    let bindings_toml_key = bindings_toml_key.to_string();
    new_pipeline()
        .convert_ir_pass::<Root>()
        .pass(modules::pass)
        .pass(rust_buffer::pass)
        .pass(rust_future::pass)
        .pass(self_types::pass)
        .pass(callable::pass)
        .pass(type_definitions_from_api::pass)
        .pass(ffi_types::pass)
        .pass(ffi_async_data::pass)
        .pass(type_nodes::pass)
        .pass(enums::pass)
        .pass(records::pass)
        .pass(objects::pass)
        .pass(callback_interfaces::pass)
        .pass(ffi_functions::pass)
        .pass(checksums::pass)
        .pass(move |root: &mut Root| rename::pass(root, &bindings_toml_key))
        .pass(sort::pass)
        .pass(default::pass)
        .pass(uniffi_traits::pass)
}
