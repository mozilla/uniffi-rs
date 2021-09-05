/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod callback_interface;
mod codetype;
mod compounds;
mod enum_;
mod error;
mod external;
mod function;
mod miscellany;
mod names;
mod object;
mod primitives;
mod record;
mod wrapper;

use crate::bindings::backend::TemplateRenderSet;
use crate::interface::*;
use crate::MergeWith;
use askama::Template;
use codetype::KotlinCodeType;
use function::KotlinCodeFunction;
use names::KotlinCodeName;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::hash::Hash;

// Some config options for it the caller wants to customize the generated Kotlin.
// Note that this can only be used to control details of the Kotlin *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    package_name: Option<String>,
    cdylib_name: Option<String>,
}

impl Config {
    pub fn package_name(&self) -> String {
        if let Some(package_name) = &self.package_name {
            package_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(format!("uniffi.{}", ci.namespace())),
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            package_name: self.package_name.merge_with(&other.package_name),
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
        }
    }
}

/// Code to be rendered in the Kotlin bindings
#[derive(Default)]
struct CodeBuilder {
    // Templates to render in the main code section
    pub code_blocks: TemplateRenderSet,
    // Modules to import.  Use a BTreeSet to make them sorted and alphabetical.
    pub imports: BTreeSet<String>,
    // Lines of code to run once we load the shared library.
    pub initialization_code_set: BTreeSet<String>,
}

impl CodeBuilder {
    pub fn code_block(mut self, template: impl 'static + Template + Hash) -> Self {
        self.code_blocks
            .insert(template)
            .expect("Error rendering templates");
        self
    }

    pub fn import(mut self, s: String) -> Self {
        self.imports.insert(s);
        self
    }

    pub fn initialization_code(mut self, s: String) -> Self {
        self.initialization_code_set.insert(s);
        self
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "wrapper.kt")]
pub struct KotlinWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    initialization_code: Vec<String>,
    code_blocks: Vec<String>,
    imports: Vec<String>,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        let mut code_builder = Default::default();
        // BTreeSet sorts the types for nicer output
        for type_ in ci.iter_types().iter().collect::<BTreeSet<_>>().iter() {
            code_builder = type_.declare_code(code_builder, ci);
        }
        for func in ci.iter_function_definitions() {
            code_builder = func.declare_code(code_builder, ci);
        }

        Self {
            config,
            ci,
            imports: code_builder.imports.into_iter().collect(),
            code_blocks: code_builder.code_blocks.into_iter().collect(),
            initialization_code: code_builder.initialization_code_set.into_iter().collect(),
        }
    }
}
