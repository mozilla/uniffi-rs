/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod callables;
mod context;
mod defaults;
mod enums;
mod interfaces;
mod names;
mod nodes;
mod packages;
mod records;
mod types;

use crate::Config;
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
