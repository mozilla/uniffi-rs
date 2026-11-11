/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Resolve the `bindings.swift.async_cancellable` list from the TOML configuration.

use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::interface::{Callable, ComponentInterface, Constructor, Method};

/// Map each name in `async_cancellable` to the name of its scaffolding function.
///
/// Names are the Rust/UDL ones (before any `rename` is applied), like `exclude` uses.
/// Unknown names, sync callables and methods of foreign-implementable traits are errors, so a
/// typo can't silently leave a callable uncancellable.
pub fn resolve(ci: &ComponentInterface, names: &[String]) -> Result<HashSet<String>> {
    names
        .iter()
        .map(|name| {
            let Some(callable) = find_callable(ci, name)? else {
                bail!("bindings.swift.async_cancellable: no function, method or constructor named `{name}`");
            };
            if !callable.is_async() {
                bail!("bindings.swift.async_cancellable: `{name}` is not async");
            }
            Ok(callable.ffi_func().name().to_string())
        })
        .collect()
}

fn find_callable<'a>(ci: &'a ComponentInterface, name: &str) -> Result<Option<&'a dyn Callable>> {
    let Some((type_name, member_name)) = name.split_once('.') else {
        return Ok(ci.get_function_definition(name).map(as_callable));
    };
    if let Some(obj) = ci.get_object_definition(type_name) {
        let member = find_member(obj.constructors(), obj.methods(), member_name);
        // Swift implementations of these traits are called through the same protocol, and that
        // call path doesn't handle the extra `throws`.
        if member.is_some() && obj.has_callback_interface() {
            bail!("bindings.swift.async_cancellable: `{name}`: methods of traits with foreign implementations can't be cancellable");
        }
        Ok(member)
    } else if let Some(rec) = ci.get_record_definition(type_name) {
        Ok(find_member(rec.constructors(), rec.methods(), member_name))
    } else if let Some(en) = ci.get_enum_definition(type_name) {
        Ok(find_member(en.constructors(), en.methods(), member_name))
    } else {
        Ok(None)
    }
}

fn find_member<'a>(
    constructors: impl IntoIterator<Item = &'a Constructor>,
    methods: impl IntoIterator<Item = &'a Method>,
    name: &str,
) -> Option<&'a dyn Callable> {
    let constructor = constructors.into_iter().find(|c| c.name() == name);
    let method = methods.into_iter().find(|m| m.name() == name);
    constructor.map(as_callable).or(method.map(as_callable))
}

fn as_callable(callable: &impl Callable) -> &dyn Callable {
    callable
}

#[cfg(test)]
mod test {
    use super::super::Config;
    use super::*;

    const UDL: &str = r#"
        namespace test {
            [Async] boolean pending();
            boolean not_async();
        };
        interface Obj {
            [Async] constructor();
            [Async] boolean pending();
            [Async] boolean other();
        };
        [Trait, WithForeign]
        interface ForeignTrait {
            [Async] boolean pending();
        };
    "#;

    fn resolve_config(toml: &str) -> Result<Config> {
        let ci = ComponentInterface::from_webidl(UDL, "crate_name")?;
        let mut config: Config = toml::from_str(toml)?;
        config.resolve_async_cancellable(&ci)?;
        Ok(config)
    }

    #[test]
    fn resolves_to_scaffolding_functions() {
        let ci = ComponentInterface::from_webidl(UDL, "crate_name").unwrap();
        let config =
            resolve_config(r#"async_cancellable = ["pending", "Obj.new", "Obj.pending"]"#).unwrap();

        let func = ci.get_function_definition("pending").unwrap();
        let obj = ci.get_object_definition("Obj").unwrap();
        let cons = obj.primary_constructor().unwrap();
        assert!(config.is_async_cancellable(func.ffi_func().name()));
        assert!(config.is_async_cancellable(cons.ffi_func().name()));
        assert!(config.is_async_cancellable(obj.get_method("pending").ffi_func().name()));
        assert!(!config.is_async_cancellable(obj.get_method("other").ffi_func().name()));
    }

    #[test]
    fn rejects_unknown_sync_and_foreign_trait_callables() {
        let error_for = |name: &str| {
            resolve_config(&format!(r#"async_cancellable = ["{name}"]"#))
                .unwrap_err()
                .to_string()
        };
        for name in ["missing", "Obj.missing", "Missing.pending"] {
            assert!(error_for(name).contains(&format!(
                "no function, method or constructor named `{name}`"
            )));
        }
        assert!(error_for("not_async").contains("`not_async` is not async"));
        assert!(error_for("ForeignTrait.pending")
            .contains("methods of traits with foreign implementations can't be cancellable"));
    }
}
