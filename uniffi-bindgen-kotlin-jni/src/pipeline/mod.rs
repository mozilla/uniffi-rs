/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod callables;
mod callbacks;
mod context;
mod defaults;
mod enums;
mod interfaces;
mod names;
mod nodes;
mod packages;
mod records;
mod types;

use crate::config::{Config, CustomTypeConfig};
use anyhow::{anyhow, bail, Result};
use context::Context;
use heck::{ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexSet;
use uniffi_bindgen::pipeline::{general, initial};
use uniffi_pipeline::{MapNode, Node, Pipeline};

pub use initial::Root as InitialRoot;
pub use nodes::*;

pub fn pipeline() -> Pipeline<initial::Root, Root> {
    general::pipeline("kotlin").pass::<Root, Context>(Context::default())
}

pub fn pipeline_for_scaffolding(package_name: String) -> Pipeline<initial::Root, Root> {
    let context = Context {
        scaffolding_crate_name: Some(package_name.replace("-", "_")),
        ..Context::default()
    };
    general::pipeline("kotlin").pass::<Root, Context>(context)
}
