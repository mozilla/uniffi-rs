/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use anyhow::{anyhow, bail, Result};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use uniffi_pipeline::Node;

// config options to customize the generated Kotlin.
#[derive(Debug, Default, Clone, Node, Serialize, Deserialize)]
pub struct Config {
    pub(super) package_name: Option<String>,
    pub(super) cdylib_name: Option<String>,
    generate_immutable_records: Option<bool>,
    #[serde(default)]
    mutable_records: IndexSet<String>,
    #[serde(default)]
    omit_checksums: bool,
    #[serde(default)]
    custom_types: IndexMap<String, CustomTypeConfig>,
    #[serde(default)]
    pub(super) external_packages: IndexMap<String, String>,
    #[serde(default)]
    android: bool,
    #[serde(default)]
    android_cleaner: Option<bool>,
    #[serde(default)]
    kotlin_target_version: Option<String>,
    #[serde(default)]
    disable_java_cleaner: bool,
    #[serde(default)]
    pub(super) exclude: Vec<String>,
}

impl Config {
    pub fn from_toml(config_toml: Option<String>) -> Result<Self> {
        Ok(match config_toml {
            Some(s) => toml::from_str(&s)?,
            None => Self::default(),
        })
    }

    pub(crate) fn android_cleaner(&self) -> bool {
        self.android_cleaner.unwrap_or(self.android)
    }

    pub(crate) fn use_enum_entries(&self) -> bool {
        self.get_kotlin_version() >= KotlinVersion::new(1, 9, 0)
    }

    /// Returns a `Version` with the contents of `kotlin_target_version`.
    /// If `kotlin_target_version` is not defined, version `0.0.0` will be used as a fallback.
    /// If it's not valid, this function will panic.
    fn get_kotlin_version(&self) -> KotlinVersion {
        self.kotlin_target_version
            .clone()
            .map(|v| {
                KotlinVersion::parse(&v).unwrap_or_else(|_| {
                    panic!("Provided Kotlin target version is not valid: {}", v)
                })
            })
            .unwrap_or(KotlinVersion::new(0, 0, 0))
    }

    // Get the package name for an external type
    fn external_package_name(&self, module_path: &str, namespace: Option<&str>) -> String {
        // config overrides are keyed by the crate name, default fallback is the namespace.
        let crate_name = module_path.split("::").next().unwrap();
        match self.external_packages.get(crate_name) {
            Some(name) => name.clone(),
            // If the module path is not in `external_packages`, we need to fall back to a default
            // with the namespace, which we hopefully have.  This is quite fragile, but it's
            // unreachable in library mode - all deps get an entry in `external_packages` with the
            // correct namespace.
            None => format!("uniffi.{}", namespace.unwrap_or(module_path)),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Node)]
#[serde(default)]
pub struct CustomTypeConfig {
    imports: Option<Vec<String>>,
    type_name: Option<String>,
    into_custom: String, // b/w compat alias for lift
    lift: String,
    from_custom: String, // b/w compat alias for lower
    lower: String,
}

// functions replace literal "{}" in strings with a specified value.
impl CustomTypeConfig {
    fn lift(&self, name: &str) -> String {
        let converter = if self.lift.is_empty() {
            &self.into_custom
        } else {
            &self.lift
        };
        converter.replace("{}", name)
    }
    fn lower(&self, name: &str) -> String {
        let converter = if self.lower.is_empty() {
            &self.from_custom
        } else {
            &self.lower
        };
        converter.replace("{}", name)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct KotlinVersion((u16, u16, u16));

impl KotlinVersion {
    fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self((major, minor, patch))
    }

    fn parse(version: &str) -> Result<Self> {
        let components = version
            .split('.')
            .map(|n| {
                n.parse::<u16>()
                    .map_err(|_| anyhow!("Invalid version string ({n} is not an integer)"))
            })
            .collect::<Result<Vec<u16>>>()?;

        match components.as_slice() {
            [major, minor, patch] => Ok(Self((*major, *minor, *patch))),
            [major, minor] => Ok(Self((*major, *minor, 0))),
            [major] => Ok(Self((*major, 0, 0))),
            _ => bail!("Invalid version string (expected 1-3 components): {version}"),
        }
    }
}
