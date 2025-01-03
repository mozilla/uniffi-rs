/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The set of all [`Type`]s used in a component interface is represented by a `TypeUniverse`,
//! which can be used by the bindings generator code to determine what type-related helper
//! functions to emit for a given component.
//!
use anyhow::{Context, Result};
use std::{collections::hash_map::Entry, collections::BTreeSet, collections::HashMap};

pub use uniffi_meta::{AsType, NamespaceMetadata, ObjectImpl, Type, TypeIterator};

/// The set of all possible types used in a particular component interface.
///
/// Every component API uses a finite number of types, including primitive types, API-defined
/// types like records and enums, and recursive types such as sequences of the above. Our
/// component API doesn't support fancy generics so this is a finitely-enumerable set, which
/// is useful to be able to operate on explicitly.
///
/// You could imagine this struct doing some clever interning of names and so-on in future,
/// to reduce the overhead of passing around [Type] instances. For now we just do a whole
/// lot of cloning.
#[derive(Clone, Debug, Default)]
pub(crate) struct TypeUniverse {
    /// The unique prefixes that we'll use for namespacing when exposing this component's API.
    pub namespace: NamespaceMetadata,
    pub namespace_docstring: Option<String>,

    // Named type definitions (including aliases).
    pub(super) type_definitions: HashMap<String, Type>,
    // All the types in the universe, by canonical type name, in a well-defined order.
    pub(super) all_known_types: BTreeSet<Type>,
}

impl TypeUniverse {
    pub fn new(namespace: NamespaceMetadata) -> Self {
        Self {
            namespace,
            ..Default::default()
        }
    }

    /// Add the definition of a named [Type].
    fn add_type_definition(&mut self, name: &str, type_: &Type) -> Result<()> {
        match self.type_definitions.entry(name.to_string()) {
            Entry::Occupied(o) => {
                // mismatched types are bad.
                let cur = o.get();
                anyhow::ensure!(
                    type_ == cur,
                    "conflicting types:\ncur: {cur:?}\nnew: {type_:?}",
                );
                Ok(())
            }
            Entry::Vacant(e) => {
                e.insert(type_.clone());
                Ok(())
            }
        }
    }

    /// Get the [Type] corresponding to a given name, if any.
    pub(super) fn get_type_definition(&self, name: &str) -> Option<Type> {
        self.type_definitions.get(name).cloned()
    }

    /// Add a [Type] to the set of all types seen in the component interface.
    pub fn add_known_type(&mut self, type_: &Type) -> Result<()> {
        // Types are more likely to already be known than not, so avoid unnecessary cloning.
        if !self.all_known_types.contains(type_) {
            self.all_known_types.insert(type_.to_owned());
        }
        // all sub-types, but we want to skip this type as we just added it above.
        for sub in type_.iter_nested_types() {
            self.add_known_type(sub)?;
        }
        if let Some(name) = type_.name() {
            self.add_type_definition(name, type_)
                .with_context(|| format!("adding named type {name}"))?;
        }
        Ok(())
    }

    /// Add many [`Type`]s...
    pub fn add_known_types(&mut self, types: TypeIterator<'_>) -> Result<()> {
        for t in types {
            self.add_known_type(t)
                .with_context(|| format!("adding type {t:?}"))?
        }
        Ok(())
    }

    pub fn is_external(&self, t: &Type) -> bool {
        t.module_path()
            .map(|p| p != self.namespace.crate_name)
            .unwrap_or(false)
    }

    #[cfg(test)]
    /// Check if a [Type] is present
    pub fn contains(&self, type_: &Type) -> bool {
        self.all_known_types.contains(type_)
    }

    pub fn iter_local_types(&self) -> impl Iterator<Item = &Type> {
        self.filter_local_types(self.all_known_types.iter())
    }

    pub fn iter_external_types(&self) -> impl Iterator<Item = &Type> {
        self.all_known_types.iter().filter(|t| self.is_external(t))
    }

    // Keep only the local types in an iterator of types.
    pub fn filter_local_types<'a>(
        &'a self,
        types: impl Iterator<Item = &'a Type>,
    ) -> impl Iterator<Item = &'a Type> {
        types.filter(|t| !self.is_external(t))
    }
}

#[cfg(test)]
mod test_type_universe {
    // All the useful functionality of the `TypeUniverse` struct
    // is tested as part of the `TypeFinder` and `TypeResolver` test suites.
}
