/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};

use super::*;

#[derive(Clone)]
pub struct Context {
    // TOML key used for the current language
    pub bindings_toml_key: String,
    pub current_namespace_name: Option<String>,
    pub current_crate_name: Option<String>,
    pub current_type: Option<Type>,
    pub current_variant: Option<initial::Variant>,
    pub current_arg_or_field_type: Option<Type>,
    pub names_used_as_error: HashSet<String>,
    // Maps namespaces to rename tables from the TOML config
    pub rename_tables: HashMap<String, toml::Table>,
    // Maps namespaces to exclude tables from the TOML config
    pub exclude_sets: HashMap<String, HashSet<String>>,
}

impl Context {
    pub fn new(bindings_toml_key: &str) -> Self {
        Self {
            bindings_toml_key: bindings_toml_key.to_string(),
            current_namespace_name: None,
            current_crate_name: None,
            current_type: None,
            current_variant: None,
            current_arg_or_field_type: None,
            names_used_as_error: HashSet::default(),
            rename_tables: HashMap::default(),
            exclude_sets: HashMap::default(),
        }
    }

    pub fn namespace_name(&self) -> Result<String> {
        self.current_namespace_name
            .clone()
            .ok_or_else(|| anyhow!("Context.current_namespace_name not set"))
    }

    pub fn crate_name(&self) -> Result<String> {
        self.current_crate_name
            .clone()
            .ok_or_else(|| anyhow!("Context.crate_name not set"))
    }

    pub fn current_type_name(&self) -> Result<String> {
        let ty = self
            .current_type
            .as_ref()
            .ok_or_else(|| anyhow!("Context.current_type not set"))?;
        match ty {
            Type::Record { name, .. }
            | Type::Enum { name, .. }
            | Type::Interface { name, .. }
            | Type::CallbackInterface { name, .. }
            | Type::Custom { name, .. } => Ok(name.to_string()),
            _ => bail!("Context.current_type_name: Invalid type ({ty:?})"),
        }
    }

    pub fn self_type(&self) -> Result<TypeNode> {
        let ty = self
            .current_type
            .clone()
            .ok_or_else(|| anyhow!("Context.current_type not set"))?;
        <Type as MapNode<TypeNode, Self>>::map_node(ty, self)
    }

    pub fn current_arg_or_field_type(&self) -> Result<TypeNode> {
        let ty = self
            .current_arg_or_field_type
            .clone()
            .ok_or_else(|| anyhow!("Context.current_arg_or_field_type not set"))?;
        <Type as MapNode<TypeNode, Self>>::map_node(ty, self)
    }

    pub fn new_name_from_rename_table(
        &self,
        namespace: &str,
        name: &str,
    ) -> Result<Option<String>> {
        match self.rename_tables.get(namespace) {
            None => bail!("Context.rename_table not set"),
            Some(rename_table) => Ok(rename_table
                .get(name)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())),
        }
    }

    pub fn update_from_root(&mut self, root: &initial::Root) -> Result<()> {
        for namespace in root.namespaces.values() {
            let rename_table = rename::extract_rename_table(namespace, &self.bindings_toml_key)?;
            let exclude_set = exclude::extract_exclude_set(namespace, &self.bindings_toml_key)?;
            self.rename_tables
                .insert(namespace.name.clone(), rename_table);
            self.exclude_sets
                .insert(namespace.name.clone(), exclude_set);
        }
        Ok(())
    }

    pub fn update_from_namespace(&mut self, namespace: &initial::Namespace) -> Result<()> {
        self.current_namespace_name = Some(namespace.name.clone());
        self.current_crate_name = Some(namespace.crate_name.clone());
        self.populate_names_used_as_error(namespace)?;
        Ok(())
    }

    fn populate_names_used_as_error(&mut self, namespace: &initial::Namespace) -> Result<()> {
        namespace.try_visit(|func: &initial::Function| self.update_from_throws(&func.throws))?;
        namespace.try_visit(|meth: &initial::Method| self.update_from_throws(&meth.throws))?;
        namespace.try_visit(|cons: &initial::Constructor| self.update_from_throws(&cons.throws))?;
        // Enums with `EnumShape::Error` are always considered errors, even if they're not directly
        // used as errors in the interface.  See the `FlatInner` error from the `error-types` fixture
        // for an example.  It's not totally clear that this is correct, but this is how things have
        // historically worked.
        namespace.visit(|en: &initial::Enum| {
            if matches!(en.shape, EnumShape::Error { .. }) {
                self.names_used_as_error.insert(en.name.clone());
            }
        });
        Ok(())
    }

    fn update_from_throws(&mut self, throws: &Option<Type>) -> Result<()> {
        if let Some(ty) = throws {
            let type_name = match ty {
                Type::Interface { name, .. }
                | Type::Record { name, .. }
                | Type::Enum { name, .. }
                | Type::CallbackInterface { name, .. }
                | Type::Custom { name, .. } => name.to_string(),
                _ => bail!("Invalid throws type: {ty:?}"),
            };
            self.names_used_as_error.insert(type_name);
        }
        Ok(())
    }

    pub fn update_from_record(&mut self, rec: &initial::Record) -> Result<()> {
        self.current_type = Some(Type::Record {
            namespace: self.namespace_name()?,
            name: rec.name.clone(),
        });
        Ok(())
    }

    pub fn update_from_enum(&mut self, en: &initial::Enum) -> Result<()> {
        self.current_type = Some(Type::Enum {
            namespace: self.namespace_name()?,
            name: en.name.clone(),
        });
        Ok(())
    }

    pub fn update_from_variant(&mut self, v: &initial::Variant) -> Result<()> {
        self.current_variant = Some(v.clone());
        Ok(())
    }

    pub fn variant(&mut self) -> Result<&initial::Variant> {
        self.current_variant
            .as_ref()
            .ok_or_else(|| anyhow!("Context.variant not set"))
    }

    pub fn update_from_interface(&mut self, int: &initial::Interface) -> Result<()> {
        self.current_type = Some(Type::Interface {
            namespace: self.namespace_name()?,
            name: int.name.clone(),
            imp: int.imp,
        });
        Ok(())
    }

    pub fn update_from_callback_interface(
        &mut self,
        cbi: &initial::CallbackInterface,
    ) -> Result<()> {
        self.current_type = Some(Type::CallbackInterface {
            namespace: self.namespace_name()?,
            name: cbi.name.clone(),
        });
        Ok(())
    }

    pub fn update_from_custom_type(&mut self, custom: &initial::CustomType) -> Result<()> {
        self.current_type = Some(Type::Custom {
            namespace: self.namespace_name()?,
            name: custom.name.clone(),
            builtin: Box::new(custom.builtin.clone()),
        });
        Ok(())
    }

    pub fn update_from_arg(&mut self, arg: &initial::Argument) -> Result<()> {
        self.current_arg_or_field_type = Some(arg.ty.clone());
        Ok(())
    }

    pub fn update_from_field(&mut self, field: &initial::Field) -> Result<()> {
        self.current_arg_or_field_type = Some(field.ty.clone());
        Ok(())
    }

    pub fn type_is_used_as_error(&self, ty: &Type) -> bool {
        match ty {
            Type::Interface { name, .. }
            | Type::Record { name, .. }
            | Type::Enum { name, .. }
            | Type::CallbackInterface { name, .. }
            | Type::Custom { name, .. } => self.names_used_as_error.contains(name),
            _ => false,
        }
    }
}
