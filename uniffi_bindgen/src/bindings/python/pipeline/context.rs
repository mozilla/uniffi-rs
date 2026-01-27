/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

#[derive(Default, Clone)]
pub struct Context {
    pub cdylib: Option<String>,
    pub current_config: Option<PythonConfig>,
    pub module_namespace: Option<String>,
}

impl Context {
    pub fn update_from_root(&mut self, root: &general::Root) {
        self.cdylib = root.cdylib.clone();
    }

    pub fn update_from_namespace(&mut self, namespace: &general::Namespace) -> Result<()> {
        self.current_config = Some(match &namespace.config_toml {
            Some(toml) => PythonConfig::from_uniffi_toml(toml)?,
            None => PythonConfig::default(),
        });
        self.module_namespace = Some(namespace.name.clone());
        Ok(())
    }

    pub fn module_namespace(&self) -> Result<&str> {
        self.module_namespace
            .as_deref()
            .ok_or_else(|| anyhow!("Context.module_namespace not set"))
    }

    pub fn config(&self) -> Result<&PythonConfig> {
        self.current_config
            .as_ref()
            .ok_or_else(|| anyhow!("Context.config not set"))
    }

    pub fn cdylib(&self) -> Result<String> {
        let default_cdylib = self.cdylib.as_deref().unwrap_or("uniffi");

        Ok(match &self.config()?.cdylib_name {
            Some(cdylib) => cdylib.clone(),
            None => default_cdylib.to_string(),
        })
    }

    pub fn external_package_name(&self, namespace: &str) -> Result<Option<String>> {
        match &self.module_namespace {
            None => bail!("Config.module_namespace not set"),
            Some(current_namespace) if current_namespace == namespace => return Ok(None),
            _ => (),
        }
        let config = self.config()?;

        Ok(Some(match config.external_packages.get(namespace) {
            None => namespace.to_string(),
            Some(package_name) if package_name.is_empty() => namespace.to_string(),
            Some(package_name) => package_name.clone(),
        }))
    }

    pub fn custom_type_config(
        &self,
        custom: &general::CustomType,
    ) -> Result<Option<CustomTypeConfig>> {
        Ok(self.config()?.custom_types.get(&custom.name).cloned())
    }
}
