/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    pub cdylib: Option<String>,
    pub module_namespace: Option<String>,
    pub recursive_type_names: HashSet<String>,
    pub builtin_types: Option<BuiltinTypes>,
    pub checksum_mode: Option<ChecksumMode>,
    pub configs: HashMap<String, PythonConfig>,
}

impl Context {
    pub fn update_from_root(&mut self, root: &general::Root) -> Result<()> {
        self.cdylib = root.cdylib.clone();
        self.checksum_mode = Some(root.checksum_mode.clone());
        self.builtin_types = Some(root.builtin_types.clone().map_node(self)?);
        self.configs = root
            .namespaces
            .values()
            .map(|namespace| {
                let config = match &namespace.config_toml {
                    Some(toml) => PythonConfig::from_uniffi_toml(toml)?,
                    None => PythonConfig::default(),
                };
                Ok((namespace.name.clone(), config))
            })
            .collect::<Result<_>>()?;
        Ok(())
    }

    pub fn update_from_namespace(&mut self, namespace: &general::Namespace) -> Result<()> {
        self.module_namespace = Some(namespace.name.clone());
        let mut recursive_type_names = HashSet::new();
        namespace.visit(|e: &general::Enum| {
            if e.recursive {
                recursive_type_names.insert(e.name.clone());
            }
        });
        namespace.visit(|r: &general::Record| {
            if r.recursive {
                recursive_type_names.insert(r.name.clone());
            }
        });
        self.recursive_type_names = recursive_type_names;
        Ok(())
    }

    pub fn builtin_types(&self) -> Result<BuiltinTypes> {
        self.builtin_types
            .clone()
            .ok_or_else(|| anyhow!("Context.builtin_types not set"))
    }

    pub fn is_recursive(&self, name: &str) -> bool {
        self.recursive_type_names.contains(name)
    }

    pub fn module_namespace(&self) -> Result<&str> {
        self.module_namespace
            .as_deref()
            .ok_or_else(|| anyhow!("Context.module_namespace not set"))
    }

    /// Get the Python module name for a namespace
    pub fn module_name<'a>(&'a self, namespace: &'a str) -> Result<&'a str> {
        Ok(self
            .config_for_namespace(namespace)?
            .module_name
            .as_deref()
            .unwrap_or(namespace))
    }

    pub fn config(&self) -> Result<&PythonConfig> {
        self.config_for_namespace(self.module_namespace()?)
    }

    pub fn config_for_namespace(&self, namespace: &str) -> Result<&PythonConfig> {
        self.configs
            .get(namespace)
            .ok_or_else(|| anyhow!("config map missing entry for {namespace}"))
    }

    pub fn cdylib(&self) -> Result<String> {
        let default_cdylib = self.cdylib.as_deref().unwrap_or("uniffi");

        Ok(match &self.config()?.cdylib_name {
            Some(cdylib) => cdylib.clone(),
            None => default_cdylib.to_string(),
        })
    }

    pub fn checksum_mode(&self) -> Result<&ChecksumMode> {
        self.checksum_mode
            .as_ref()
            .ok_or_else(|| anyhow!("Context.checksum_mode not set"))
    }

    pub fn external_package_name(&self, namespace: &str) -> Result<Option<String>> {
        match &self.module_namespace {
            None => bail!("Config.module_namespace not set"),
            Some(current_namespace) if current_namespace == namespace => return Ok(None),
            _ => (),
        }
        let config = self.config()?;

        Ok(Some(match config.external_packages.get(namespace) {
            None => self.module_name(namespace)?.to_string(),
            Some(package_name) if package_name.is_empty() => {
                self.module_name(namespace)?.to_string()
            }
            Some(package_name) => package_name.clone(),
        }))
    }

    pub fn custom_type_config(
        &self,
        custom: &general::CustomType,
    ) -> Result<Option<CustomTypeConfig>> {
        Ok(self.config()?.custom_types.get(&custom.orig_name).cloned())
    }
}
