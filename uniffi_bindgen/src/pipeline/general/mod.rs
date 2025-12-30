/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! General IR - a useful starting point for language-specific IRs.

mod callable;
mod callback_interfaces;
mod checksums;
mod context;
mod default;
mod enums;
mod ffi_async_data;
mod ffi_functions;
mod ffi_types;
mod namespaces;
mod nodes;
mod objects;
mod records;
mod rename;
mod rust_buffer;
mod rust_future;
mod sort;
mod type_definitions_from_api;
mod types;
mod uniffi_traits;
use super::initial;
use anyhow::{anyhow, bail, Result};
pub use context::Context;
pub use indexmap::{IndexMap, IndexSet};
pub use nodes::*;
use uniffi_pipeline::{new_pipeline, use_prev_node, MapNode, Node, Pipeline};

/// General IR pipeline
///
/// This is the shared beginning for all bindings pipelines.
/// Bindings generators will define their own pipeline based on the output of this one.
pub fn pipeline(bindings_toml_key: &str) -> Pipeline<initial::Root, Root> {
    new_pipeline().pass::<Root, Context>(Context::new(bindings_toml_key))
}
