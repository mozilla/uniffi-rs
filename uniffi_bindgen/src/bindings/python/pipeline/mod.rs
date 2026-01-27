/* This Source Code Form is subject to the terms of the Mozilla Publicpypimod
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{anyhow, bail, Result};
use indexmap::{IndexMap, IndexSet};

// pub use crate::pipeline::{general, initial};
use uniffi_pipeline::{use_prev_node, MapNode, Node, Pipeline};
mod callables;
mod callback_interfaces;
mod config;
mod context;
mod default;
mod enums;
mod error;
mod ffi_types;
mod interfaces;
mod modules;
mod names;
pub mod nodes;
mod types;

pub use config::*;
pub use context::Context;
pub use nodes::*;
//
// For now, this is just the general pipeline.
// Defining this allows us to use the pipeline CLI to inspect the general pipeline.

pub use crate::pipeline::{general, initial};

pub fn pipeline() -> Pipeline<initial::Root, Root> {
    general::pipeline("python").pass::<Root, Context>(Context::default())
}
