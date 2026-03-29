/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod callables;
mod context;
mod ffi_types;
mod names;
mod nodes;
mod packages;
mod root;
mod types;

use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use context::Context;
use heck::{ToLowerCamelCase, ToUpperCamelCase};
use indexmap::IndexSet;
use uniffi_bindgen::pipeline::{general, initial};
use uniffi_pipeline::{MapNode, Node, Pipeline};

use crate::Config;
pub use initial::Root as InitialRoot;
pub use nodes::*;

pub fn pipeline() -> Pipeline<initial::Root, Root> {
    general::pipeline("kotlin").pass::<Root, Context>(Context::default())
}

pub fn pipeline_for_scaffolding(pkg_name: String) -> Pipeline<initial::Root, Root> {
    let context = Context {
        scaffolding_crate_name: Some(pkg_name.replace("-", "_")),
        ..Context::default()
    };

    general::pipeline("kotlin").pass::<Root, Context>(context)
}
