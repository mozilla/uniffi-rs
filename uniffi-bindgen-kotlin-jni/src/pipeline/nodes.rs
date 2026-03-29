/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

uniffi_pipeline::use_prev_node!(general::ObjectImpl);
uniffi_pipeline::use_prev_node!(general::Type);

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
#[map_node(callables::map_callable)]
pub struct Callable {
    pub kind: CallableKind,
    pub name: String,
    pub arguments: Vec<Argument>,
    pub return_type: Option<TypeNode>,
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::CallableKind))]
pub enum CallableKind {
    Function,
    Method {
        self_type: TypeNode,
    },
    Constructor {
        self_type: TypeNode,
        primary: bool,
    },
    VTableMethod {
        self_type: TypeNode,
        for_callback_interface: bool,
    },
}

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Argument))]
pub struct Argument {
    pub name: String,
    pub ty: TypeNode,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Node, MapNode)]
#[map_node(from(general::TypeNode))]
#[map_node(types::map_type_node)]
pub struct TypeNode {
    pub canonical_name: String,
    pub is_used_as_error: bool,
    pub ty: Type,
    pub type_kt: String,
    pub read_fn_rs: String,
    pub write_fn_rs: String,
    pub read_fn_kt: String,
    pub write_fn_kt: String,
    // Note: no ffi_type field, we have a very different FFI than the general IR
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

    pub fn name_jni(&self) -> String {
        self.name.replace(".", "/")
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

    pub fn return_type_kt(&self) -> &str {
        match &self.return_type {
            None => "Unit",
            Some(ty) => &ty.type_kt,
        }
    }
}

impl Argument {
    pub fn name_kt(&self) -> String {
        format!("`{}`", self.name.to_lower_camel_case())
    }

    pub fn name_rs(&self) -> String {
        names::escape_rust(&self.name)
    }
}
