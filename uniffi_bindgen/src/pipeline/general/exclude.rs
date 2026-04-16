/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Exclude items based on TOML table lookups.
//! Items can only be renamed in their own crate, so we use uniffi.toml from each crate
//! Then skip items based on these lookups.

use std::collections::HashSet;

use super::*;

pub fn extract_exclude_set(
    namespace: &initial::Namespace,
    bindings_toml_key: &str,
) -> Result<HashSet<String>> {
    let Some(config_toml) = &namespace.config_toml else {
        return Ok(HashSet::default());
    };
    let config: toml::Table = toml::from_str(config_toml)?;
    let exclude = config
        .get("bindings")
        .and_then(|b| b.as_table())
        .and_then(|b| b.get(bindings_toml_key))
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("exclude"))
        .cloned();

    Ok(match exclude {
        Some(toml) => toml.try_into()?,
        None => HashSet::default(),
    })
}

pub fn should_exclude_toplevel_item(name: &str, context: &Context) -> Result<bool> {
    let Some(exclude_set) = context.exclude_sets.get(&context.namespace_name()?) else {
        return Ok(false);
    };
    Ok(exclude_set.contains(name))
}

pub fn should_exclude_method(method_name: &str, context: &Context) -> Result<bool> {
    let Some(exclude_set) = context.exclude_sets.get(&context.namespace_name()?) else {
        return Ok(false);
    };
    let key = format!("{}.{method_name}", context.current_type_name()?);
    Ok(exclude_set.contains(&key))
}
