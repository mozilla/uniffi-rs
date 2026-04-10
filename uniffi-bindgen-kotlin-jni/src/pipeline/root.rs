/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;

pub fn map_root(input: general::Root, context: &Context) -> Result<Root> {
    let mut context = context.clone();
    context.update_from_root(&input)?;

    Ok(Root {
        cdylib: input.cdylib,
        packages: Vec::from_iter(input.namespaces.map_node(&context)?.into_values()),
    })
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

    pub fn jni_methods(&self) -> impl Iterator<Item = (&str, &Callable)> {
        self.packages.iter().flat_map(Package::jni_methods)
    }

    /// Type definitions to generate FFI functions for
    ///
    /// This de-dupes the type definitions for all packages so we only don't generate duplicate
    /// functions for types that may be used in multiple packages like `Vec<u32>`.
    pub fn ffi_type_definitions(&self) -> impl Iterator<Item = &TypeDefinition> {
        let mut seen = HashSet::new();
        self.packages
            .iter()
            .flat_map(|p| &p.type_definitions)
            .filter(move |type_def| {
                seen.insert(match type_def {
                    TypeDefinition::Record(r) => r.self_type.id,
                    TypeDefinition::Enum(e) => e.self_type.id,
                    TypeDefinition::Optional(o) => o.self_type.id,
                    TypeDefinition::Sequence(s) => s.self_type.id,
                    TypeDefinition::Map(m) => m.self_type.id,
                    TypeDefinition::Set(s) => s.self_type.id,
                    TypeDefinition::Class(c) => c.self_type.id,
                    TypeDefinition::Custom(c) => c.self_type.id,
                    TypeDefinition::CallbackInterface(c) => c.self_type.id,
                    TypeDefinition::Interface(_) => return false,
                })
            })
    }

    /// Types where we need to call the Kotlin lift function from Rust
    ///
    /// We need to do this for non-primitive types that are returned
    /// and any error types that get thrown from Rust functions.
    pub fn lift_kt_from_rust_types(&self) -> impl Iterator<Item = &TypeNode> {
        let mut seen = HashSet::new();
        let mut type_nodes = vec![];
        self.visit(|callable: &Callable| {
            if let Some(return_type) = callable.return_type() {
                if !return_type.lowers_to_primitive() && seen.insert(return_type.id) {
                    type_nodes.push(return_type);
                }
            }
            if let Some(throws_type) = callable.throws_type() {
                if seen.insert(throws_type.id) {
                    type_nodes.push(throws_type);
                }
            }
        });
        type_nodes.into_iter()
    }

    pub fn rust_async_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_rust_function() && c.is_async)
    }

    pub fn kotlin_sync_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_kotlin_function() && !c.is_async)
    }

    pub fn kotlin_async_callable_results(&self) -> impl Iterator<Item = &CallableResult> {
        self.iter_callable_results_that_match(|c| c.is_for_kotlin_function() && c.is_async)
    }

    fn iter_callable_results_that_match(
        &self,
        filter: impl Fn(&Callable) -> bool,
    ) -> impl Iterator<Item = &CallableResult> {
        let mut seen = HashSet::new();
        let mut results = vec![];
        self.visit(|callable: &Callable| {
            if filter(callable) && seen.insert(callable.result.id()) {
                results.push(&callable.result);
            }
        });
        results.into_iter()
    }

    pub fn classes(&self) -> impl Iterator<Item = &Class> {
        self.packages.iter().flat_map(Package::classes)
    }

    pub fn disable_java_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        //
        // Maybe we can leverage https://github.com/mozilla/uniffi-rs/issues/2866 to improve this
        self.packages.iter().any(|p| p.config.disable_java_cleaner)
    }

    pub fn enable_android_cleaner(&self) -> bool {
        // Try to merge the different config values as best we can.
        //
        // Maybe we can leverage https://github.com/mozilla/uniffi-rs/issues/2866 to improve this
        self.packages.iter().any(|p| p.config.android_cleaner())
    }
}
