/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Initial IR, this is the Metadata from uniffi_meta with some slight changes:
//!
//! * Crate names / modules names / namespace names are normalized to be namespace names
//! * The metadata list is grouped into a tree-like structure:
//!    * At the top is Namespace values (modules for most languages)
//!    * Namespaces have types and functions as their children
//!    * Types can have methods/constructors etc. as their children.

mod context;
mod from_uniffi_meta;
mod nodes;
mod types;

use anyhow::{anyhow, bail, Result};

pub use context::Context;
pub use from_uniffi_meta::UniffiMetaConverter;
pub use nodes::*;
pub use uniffi_pipeline::{use_prev_node, MapNode, Node};
