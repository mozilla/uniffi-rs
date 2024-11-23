/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Sort BindingsIr nodes in dependency order.
//!
//! Dependencies should come before the nodes that depend on them.
//! This avoids attribute errors for languages like Python.
//!
//! Note: this code can't handle circular dependencies correctly (but at least it won't infinitely
//! recurse).

use std::collections::HashMap;

use super::*;

pub fn sort_ffi_definitions(items: impl IntoIterator<Item = FfiDefinition>) -> Vec<FfiDefinition> {
    DependencySorter::new(items, FfiDefinitionDependencyLogic).sort()
}

pub fn sort_types(items: impl IntoIterator<Item = TypeDefinition>) -> Vec<TypeDefinition> {
    DependencySorter::new(items, TypeDefinitionDependencyLogic).sort()
}

// Generalized dependency sort using a version of depth-first topological sort:
//
// https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search
//
// Basically, we do a depth first into the dependency graph, which ensures that we get dependencies
// first.
struct DependencySorter<L: DependencyLogic> {
    logic: L,
    unsorted: HashMap<String, L::Item>,
    sorted: Vec<L::Item>,
}

impl<L: DependencyLogic> DependencySorter<L> {
    fn new(items: impl IntoIterator<Item = L::Item>, logic: L) -> Self {
        let unsorted: HashMap<_, _> = items
            .into_iter()
            .map(|i| (logic.item_name(&i), i))
            .collect();
        Self {
            unsorted,
            sorted: vec![],
            logic,
        }
    }

    fn sort(mut self) -> Vec<L::Item> {
        while let Some(name) = self.unsorted.keys().next() {
            self.recurse(name.clone());
        }
        self.sorted
    }

    fn recurse(&mut self, current_name: String) {
        let Some(current_item) = self.unsorted.remove(&current_name) else {
            // If `current_name` is not in unsorted, then we've already processed the item
            return;
        };
        // Add all dependents first
        for name in self.logic.dependency_names(&current_item) {
            self.recurse(name);
        }
        // Then add the current item
        self.sorted.push(current_item);
    }
}

/// Logic for a particular dependency sort
trait DependencyLogic {
    // What are we sorting?
    type Item;

    // Get the name of an item
    fn item_name(&self, item: &Self::Item) -> String;

    // Get the names of an item's dependencies
    fn dependency_names(&self, item: &Self::Item) -> Vec<String>;
}

struct FfiDefinitionDependencyLogic;

impl DependencyLogic for FfiDefinitionDependencyLogic {
    type Item = FfiDefinition;

    fn item_name(&self, ffi_def: &FfiDefinition) -> String {
        ffi_def.name().to_string()
    }

    fn dependency_names(&self, ffi_def: &FfiDefinition) -> Vec<String> {
        match ffi_def {
            FfiDefinition::Struct(ffi_struct) => ffi_struct
                .fields
                .iter()
                .filter_map(|f| Self::type_dependency_name(&f.ty))
                .collect(),
            FfiDefinition::Function(func) => func
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func.return_type)
                .filter_map(Self::type_dependency_name)
                .collect(),
            FfiDefinition::FunctionType(func_type) => func_type
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func_type.return_type)
                .filter_map(Self::type_dependency_name)
                .collect(),
        }
    }
}

impl FfiDefinitionDependencyLogic {
    fn type_dependency_name(ffi_type: &FfiType) -> Option<String> {
        match &ffi_type {
            FfiType::Struct(name) | FfiType::FunctionPointer(name) => Some(name.clone()),
            FfiType::Reference(inner) | FfiType::MutReference(inner) => {
                Self::type_dependency_name(inner)
            }
            _ => None,
        }
    }
}

struct TypeDefinitionDependencyLogic;

impl DependencyLogic for TypeDefinitionDependencyLogic {
    type Item = TypeDefinition;

    fn item_name(&self, type_def: &TypeDefinition) -> String {
        type_def.canonical_name().to_string()
    }

    fn dependency_names(&self, type_def: &TypeDefinition) -> Vec<String> {
        match type_def {
            TypeDefinition::Simple(_) => vec![],
            TypeDefinition::Optional(OptionalType { inner, .. })
            | TypeDefinition::Sequence(SequenceType { inner, .. }) => {
                vec![inner.canonical_name()]
            }
            TypeDefinition::Map(MapType { key, value, .. }) => {
                vec![key.canonical_name(), value.canonical_name()]
            }
            TypeDefinition::Record(r) => r.fields.iter().map(|f| f.ty.canonical_name()).collect(),
            TypeDefinition::Enum(e) => e
                .variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| f.ty.canonical_name()))
                .collect(),
            TypeDefinition::Interface(i) => i
                .trait_impls
                .iter()
                .map(|i| format!("Type{}", i.trait_name))
                .chain(i.methods.iter().flat_map(|m| {
                    m.callable
                        .arguments
                        .iter()
                        .map(|a| &a.ty)
                        .chain(&m.callable.return_type)
                        .chain(&m.callable.throws_type)
                        .map(|ty| ty.canonical_name())
                }))
                .collect(),
            TypeDefinition::CallbackInterface(c) => c
                .methods
                .iter()
                .flat_map(|m| {
                    m.callable
                        .arguments
                        .iter()
                        .map(|a| &a.ty)
                        .chain(&m.callable.return_type)
                        .chain(&m.callable.throws_type)
                })
                .map(|ty| ty.canonical_name())
                .collect(),
            TypeDefinition::Custom(custom) => {
                vec![custom.builtin.canonical_name()]
            }
            TypeDefinition::External(_) => vec![],
        }
    }
}
