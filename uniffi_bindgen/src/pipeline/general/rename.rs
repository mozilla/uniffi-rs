/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Renames items based on TOML table lookups.
//! Items can only be renamed in their own crate, so we use uniffi.toml from each crate
//! Then mutate all itens based on these lookups.

use super::*;
use std::collections::HashMap;

pub fn pass(root: &mut Root, bindings_toml_key: &str) -> Result<()> {
    let mut namespace_renames: HashMap<String, toml::Table> = HashMap::new();

    for namespace in root.namespaces.values() {
        if let Some(renames) = extract_rename_table(namespace, bindings_toml_key)? {
            namespace_renames.insert(namespace.name.clone(), renames);
        }
    }

    if namespace_renames.is_empty() {
        return Ok(());
    }

    for namespace in root.namespaces.values_mut() {
        apply_renames(namespace, &namespace_renames)?;
    }

    Ok(())
}

fn extract_rename_table(
    namespace: &Namespace,
    bindings_toml_key: &str,
) -> Result<Option<toml::Table>> {
    let Some(config_toml) = &namespace.config_toml else {
        return Ok(None);
    };
    let config: toml::Table = toml::from_str(config_toml)?;

    Ok(config
        .get("bindings")
        .and_then(|b| b.as_table())
        .and_then(|b| b.get(bindings_toml_key))
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("rename"))
        .and_then(|r| r.as_table())
        .cloned())
}

fn apply_renames(
    namespace: &mut Namespace,
    namespace_renames: &HashMap<String, toml::Table>,
) -> Result<()> {
    let ns = namespace.name.clone();

    namespace.visit_mut(|callable: &mut Callable| {
        rename_callable(callable, &ns, namespace_renames);
    });

    // rename all the types and any sub-elements in them.
    for type_def in namespace.type_definitions.iter_mut() {
        match type_def {
            TypeDefinition::Interface(interface) => {
                if let Some(new_name) = new_name(&ns, &interface.name, namespace_renames) {
                    interface.name = new_name;
                }
            }
            TypeDefinition::CallbackInterface(callback) => {
                if let Some(new_name) = new_name(&ns, &callback.name, namespace_renames) {
                    callback.name = new_name;
                }
            }
            TypeDefinition::Record(record) => {
                rename_record(record, &ns, namespace_renames);
            }
            TypeDefinition::Enum(enum_) => {
                rename_enum(enum_, &ns, namespace_renames);
            }
            _ => {}
        }
    }

    rename_types(namespace, namespace_renames);

    Ok(())
}

fn new_name(namespace: &str, name: &str, renames: &HashMap<String, toml::Table>) -> Option<String> {
    renames
        .get(namespace)
        .and_then(|table| table.get(name))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn rename_types(ns: &mut Namespace, renames: &HashMap<String, toml::Table>) {
    ns.visit_mut(|ty: &mut Type| match ty {
        Type::Record {
            namespace, name, ..
        }
        | Type::Enum {
            namespace, name, ..
        }
        | Type::Interface {
            namespace, name, ..
        }
        | Type::Custom {
            namespace, name, ..
        } => {
            if let Some(new_name) = new_name(namespace, name, renames) {
                *name = new_name;
            }
        }
        _ => {}
    });
}

fn rename_callable(
    callable: &mut Callable,
    namespace: &str,
    renames: &HashMap<String, toml::Table>,
) {
    let fn_name = callable.name.clone();

    let callable_key = match &callable.kind {
        CallableKind::Constructor { self_type, .. }
        | CallableKind::Method { self_type }
        | CallableKind::VTableMethod { self_type, .. } => Some(self_type.ty.name().unwrap()),
        CallableKind::Function => None,
    }
    .map(|ob_prefix| format!("{ob_prefix}.{fn_name}"))
    .unwrap_or(fn_name);

    for arg in &mut callable.arguments {
        let arg_path = format!("{callable_key}.{}", arg.name);
        if let Some(new_name) = new_name(namespace, &arg_path, renames) {
            arg.name = new_name;
        }
    }

    if let Some(new_name) = new_name(namespace, &callable_key, renames) {
        callable.name = new_name;
    }
}

fn rename_record(record: &mut Record, namespace: &str, renames: &HashMap<String, toml::Table>) {
    let record_name = record.name.clone();

    for field in &mut record.fields {
        let field_path = format!("{}.{}", record_name, field.name);
        if let Some(new_name) = new_name(namespace, &field_path, renames) {
            field.name = new_name;
        }
    }

    if let Some(new_name) = new_name(namespace, &record_name, renames) {
        record.name = new_name;
    }
}

fn rename_enum(enum_: &mut Enum, namespace: &str, renames: &HashMap<String, toml::Table>) {
    let enum_name = enum_.name.clone();

    for variant in &mut enum_.variants {
        let variant_name = variant.name.clone();
        let variant_path = format!("{}.{}", enum_name, variant_name);

        for field in &mut variant.fields {
            let field_path = format!("{}.{}", variant_path, field.name);
            if let Some(new_name) = new_name(namespace, &field_path, renames) {
                field.name = new_name;
            }
        }

        if let Some(new_name) = new_name(namespace, &variant_path, renames) {
            variant.name = new_name;
        }
    }

    if let Some(new_name) = new_name(namespace, &enum_name, renames) {
        enum_.name = new_name;
    }
}
