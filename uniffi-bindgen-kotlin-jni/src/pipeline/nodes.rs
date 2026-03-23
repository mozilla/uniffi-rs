/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Root))]
pub struct Root {
    pub cdylib: Option<String>,
    #[map_node(Vec::from_iter(self.namespaces.map_node(context)?.into_values()))]
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Namespace))]
#[map_node(packages::map_namespace)]
pub struct Package {
    pub name: String,
    pub crate_name: String,
    pub config: Config,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Function))]
pub struct Function {
    #[map_node(callables::function_jni_method_name(&self, context)?)]
    pub jni_method_name: String,
    pub module_path: String,
    pub docstring: Option<String>,
    pub callable: Callable,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Callable))]
pub struct Callable {
    #[map_node(callables::map_kind(&self, context)?)]
    pub kind: CallableKind,
    pub name: String,
}

#[derive(Debug, Clone, Node, MapNode)]
pub enum CallableKind {
    Function,
}

impl Root {
    pub fn cdylib_name(&self) -> Result<String> {
        let config_names: IndexSet<_> = self
            .packages
            .iter()
            .filter_map(|p| p.config.cdylib_name.as_deref())
            .collect();
        Ok(match config_names.len() {
            0 => match &self.cdylib {
                Some(name) => name.to_string(),
                None => bail!("Unknown cdylib name.  Use `src:[crate_name]` to generate bindings or set it in a `uniffi.toml` config"),
            }
            1 => config_names.into_iter().next().unwrap().to_string(),
            _ => bail!("Conflicting cdylib names in `uniffi.toml` files: {:?}", Vec::from_iter(config_names)),
        })
    }
}

impl Package {
    pub fn jni_class(&self) -> String {
        format!("`{}`", self.crate_name.to_upper_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.crate_name)
    }
}

impl Callable {
    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }

    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }
}
