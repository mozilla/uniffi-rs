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

/// Container for code to be rendered in the Kotlin bindings
#[derive(Default)]
struct CodeDeclarations {
    // These store template renders
    pub definitions: TemplateRenderSet,
    pub runtimes: TemplateRenderSet,
    // Imports and initialization code is short.  It's easier to use strings than Templates.  Use a
    // BTreeSet to make imports nice and alphabetical.
    pub imports: BTreeSet<String>,
    pub initialization_code: BTreeSet<String>,
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "wrapper.kt")]
pub struct KotlinWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
    declarations: CodeDeclarations,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        let mut declarations = Default::default();
        // BTreeSet sorts the types for nicer output
        for type_ in ci.iter_types().iter().collect::<BTreeSet<_>>().iter() {
            type_
                .declare_code(&mut declarations, ci)
                .expect("Error rendering templates");
        }
        for func in ci.iter_function_definitions() {
            func.declare_code(&mut declarations, ci)
                .expect("Error rendering templates");
        }

        Self {
            config,
            ci,
            declarations,
        }
    }

    pub fn initialization_code(&self) -> Vec<&String> {
        self.declarations.initialization_code.iter().collect()
    }

    pub fn declaration_code(&self) -> Vec<&String> {
        self.declarations
            .runtimes
            .iter()
            .chain(self.declarations.definitions.iter())
            .collect()
    }

    pub fn imports(&self) -> Vec<&String> {
        self.declarations.imports.iter().collect()
    }
}
