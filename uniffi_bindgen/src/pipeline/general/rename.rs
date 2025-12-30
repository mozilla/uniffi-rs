/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Renames items based on TOML table lookups.
//! Items can only be renamed in their own crate, so we use uniffi.toml from each crate
//! Then mutate all itens based on these lookups.

use super::*;

pub fn extract_rename_table(
    namespace: &initial::Namespace,
    bindings_toml_key: &str,
) -> Result<toml::Table> {
    let Some(config_toml) = &namespace.config_toml else {
        return Ok(toml::Table::default());
    };
    let config: toml::Table = toml::from_str(config_toml)?;

    Ok(config
        .get("bindings")
        .and_then(|b| b.as_table())
        .and_then(|b| b.get(bindings_toml_key))
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("rename"))
        .and_then(|r| r.as_table())
        .cloned()
        .unwrap_or_default())
}

pub fn type_(namespace: &str, name: String, context: &Context) -> Result<String> {
    Ok(context
        .new_name_from_rename_table(namespace, &name)?
        .unwrap_or(name))
}

pub fn field(field_name: String, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    let type_name = context.current_type_name()?;
    let key = match &context.current_variant {
        None => format!("{type_name}.{field_name}"),
        Some(v) => {
            let variant_name = &v.name;
            format!("{type_name}.{variant_name}.{field_name}")
        }
    };
    Ok(context
        .new_name_from_rename_table(&namespace, &key)?
        .unwrap_or(field_name))
}

pub fn variant(variant_name: String, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    let key = format!("{}.{variant_name}", context.current_type_name()?);
    Ok(context
        .new_name_from_rename_table(&namespace, &key)?
        .unwrap_or(variant_name))
}

pub fn func(fn_name: String, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    Ok(context
        .new_name_from_rename_table(&namespace, &fn_name)?
        .unwrap_or(fn_name))
}

pub fn func_arg(arg_name: String, fn_name: &str, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    let key = format!("{fn_name}.{arg_name}");
    Ok(context
        .new_name_from_rename_table(&namespace, &key)?
        .unwrap_or(arg_name))
}

/// Get the new name for a method or constructor
pub fn method(method_name: String, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    let key = format!("{}.{method_name}", context.current_type_name()?);
    Ok(context
        .new_name_from_rename_table(&namespace, &key)?
        .unwrap_or(method_name))
}

pub fn method_arg(arg_name: String, method_name: &str, context: &Context) -> Result<String> {
    let namespace = context.namespace_name()?;
    let key = format!("{}.{method_name}.{arg_name}", context.current_type_name()?);
    Ok(context
        .new_name_from_rename_table(&namespace, &key)?
        .unwrap_or(arg_name))
}
