/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

uniffi_pipeline::use_prev_node!(general::ObjectImpl);
uniffi_pipeline::use_prev_node!(general::TraitKind);
uniffi_pipeline::use_prev_node!(general::Type);

#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::Root))]
#[map_node(root::map_root)]
pub struct Root {
    pub cdylib: Option<String>,
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
#[map_node(callables::map_function)]
pub struct Function {
    pub jni_method_name: String,
    pub docstring: Option<String>,
    pub callable: Callable,
}

#[derive(Debug, Clone, Node)]
pub struct Callable {
    pub kind: CallableKind,
    pub name: String,
    pub is_async: bool,
    pub fully_qualified_name_rs: String,
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

/// Wrap `Type` so that we can add extra fields that are set for all variants.
#[derive(Debug, Clone, Node, MapNode)]
#[map_node(from(general::TypeNode))]
pub struct TypeNode {
    pub ty: Type,
}
