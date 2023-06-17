/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The set of all [`Type`]s used in a component interface is represented by a `TypeUniverse`,
//! which can be used by the bindings generator code to determine what type-related helper
//! functions to emit for a given component.
//!
use anyhow::{bail, Result};
use std::{collections::hash_map::Entry, collections::BTreeSet, collections::HashMap};

mod finder;
pub(super) use finder::TypeFinder;
mod resolver;
pub(super) use resolver::{resolve_builtin_type, TypeResolver};

pub use uniffi_meta::{AsType, ExternalKind, ObjectImpl, Type, TypeIterator};

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
#[derive(Debug, Default)]
pub(crate) struct TypeUniverse {
    /// The unique prefix that we'll use for namespacing when exposing this component's API.
    pub namespace: String,
    // Named type definitions (including aliases).
    type_definitions: HashMap<String, Type>,
    // All the types in the universe, by canonical type name, in a well-defined order.
    all_known_types: BTreeSet<Type>,
}

impl TypeUniverse {
    /// Add the definitions of all named [Type]s from a given WebIDL definition.
    ///
    /// This will fail if you try to add a name for which an existing type definition exists.
    pub(super) fn add_type_definitions_from<T: TypeFinder>(&mut self, defn: T) -> Result<()> {
        defn.add_type_definitions_to(self)
    }

    /// Add the definition of a named [Type].
    ///
    /// This will fail if you try to add a name for which an existing type definition exists.
    pub fn add_type_definition(&mut self, name: &str, type_: Type) -> Result<()> {
        if resolve_builtin_type(name).is_some() {
            bail!("please don't shadow builtin types ({name}, {:?})", type_,);
        }
        self.add_known_type(&type_);
        match self.type_definitions.entry(name.to_string()) {
            Entry::Occupied(o) => {
                let existing_def = o.get();
                if type_ == *existing_def
                    && matches!(type_, Type::Record { .. } | Type::Enum { .. })
                {
                    // UDL and proc-macro metadata are allowed to define the same record, enum and
                    // error types, if the definitions match (fields and variants are checked in
                    // add_{record,enum,error}_definition)
                    Ok(())
                } else {
                    bail!(
                        "Conflicting type definition for `{name}`! \
                         existing definition: {existing_def:?}, \
                         new definition: {type_:?}"
                    );
                }
            }
            Entry::Vacant(e) => {
                e.insert(type_);
                Ok(())
            }
        }
    }

    /// Get the [Type] corresponding to a given name, if any.
    pub(super) fn get_type_definition(&self, name: &str) -> Option<Type> {
        self.type_definitions.get(name).cloned()
    }

    /// Get the [Type] corresponding to a given WebIDL type node.
    ///
    /// If the node is a structural type (e.g. a sequence) then this will also add
    /// it to the set of all types seen in the component interface.
    pub(crate) fn resolve_type_expression<T: TypeResolver>(&mut self, expr: T) -> Result<Type> {
        expr.resolve_type_expression(self)
    }

    /// Add a [Type] to the set of all types seen in the component interface.
    pub fn add_known_type(&mut self, type_: &Type) {
        // Types are more likely to already be known than not, so avoid unnecessary cloning.
        if !self.all_known_types.contains(type_) {
            self.all_known_types.insert(type_.to_owned());

            // Add inner types. For UDL, this is actually pointless extra work (as is calling
            // add_known_type from add_function_definition), but for the proc-macro frontend
            // this is important if the inner type isn't ever mentioned outside one of these
            // generic builtin types.
            match type_ {
                Type::Optional { inner_type } => self.add_known_type(inner_type),
                Type::Sequence { inner_type } => self.add_known_type(inner_type),
                Type::Map {
                    key_type,
                    value_type,
                } => {
                    self.add_known_type(key_type);
                    self.add_known_type(value_type);
                }
                _ => {}
            }
        }
    }

    /// Check if a [Type] is present
    pub fn contains(&self, type_: &Type) -> bool {
        self.all_known_types.contains(type_)
    }

    /// Iterator over all the known types in this universe.
    pub fn iter_known_types(&self) -> impl Iterator<Item = &Type> {
        self.all_known_types.iter()
    }
}

#[cfg(test)]
mod test_type_universe {
    // All the useful functionality of the `TypeUniverse` struct
    // is tested as part of the `TypeFinder` and `TypeResolver` test suites.
}
