/* This Source Code Form is subject to the terms of the Mozilla Publicpypimod
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{bail, Result};

pub use crate::pipeline::{general, initial};
use uniffi_pipeline::{Node, Pipeline};
mod callback_interfaces;
mod config;
mod default;
mod error;
mod external_types;
mod ffi_types;
mod interfaces;
mod modules;
mod names;
pub mod nodes;
mod types;

pub use nodes::*;

// For now, this is just the general pipeline.
// Defining this allows us to use the pipeline CLI to inspect the general pipeline.
pub fn pipeline() -> Pipeline<initial::Root, Root> {
    general::pipeline()
        .convert_ir_pass::<Root>()
        .pass(config::pass)
        .pass(external_types::pass)
        .pass(names::pass)
        .pass(interfaces::pass)
        .pass(modules::pass)
        .pass(callback_interfaces::pass)
        .pass(types::pass)
        .pass(default::pass)
        .pass(ffi_types::pass)
        .pass(error::pass)
}
