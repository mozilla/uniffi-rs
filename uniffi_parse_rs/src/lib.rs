/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use camino::Utf8Path;
pub use uniffi_meta::MetadataGroup;

mod attrs;
mod enums;
mod errors;
mod files;
mod functions;
mod impls;
mod ir;
mod items;
mod kw;
mod macros;
mod modules;
mod objects;
mod paths;
mod records;
mod traits;
mod types;
mod use_;

pub use enums::{Enum, Variant};
pub use errors::{Error, ErrorContext, ErrorKind};
pub use functions::{Argument, Function, ReturnType};
pub use impls::Impl;
pub use ir::Ir;
pub use items::{BuiltinItem, Item};
pub use macros::resolve_macros;
pub use modules::Module;
pub use objects::{Constructor, Method, Object, SelfArg};
pub use paths::RPath;
pub use records::{Field, Record};
pub use traits::Trait;

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type MetadataGroupMap = HashMap<String, MetadataGroup>;

/// Top-level API for `uniffi_parse_rs`
///
/// This parses Rust sources and creates a `MetadataGroupMap` from them.
/// All failable methods return `anyhow::Result`
/// to integrate better with the rest of the UniFFI.
pub struct Parser {
    ir: Ir,
}

impl Parser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self { ir: Ir::default() })
    }

    pub fn add_crate_root(&mut self, name: &str, path: &Utf8Path) -> anyhow::Result<&Module> {
        self.ir
            .add_crate_root(name, path)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn into_uniffi_meta(mut self) -> anyhow::Result<MetadataGroupMap> {
        resolve_macros(&mut self.ir).map_err(|e| anyhow::anyhow!("{e}"))?;
        self.ir
            .into_metadata_group_map()
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
