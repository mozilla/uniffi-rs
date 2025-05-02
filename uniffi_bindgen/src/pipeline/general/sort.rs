/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Sort definitions so that dependencies come first
//!
//! This is needed for languages like Python that will throw errors if the dependent type is
//! defined by its dependency.

use super::*;

pub fn pass(module: &mut Module) -> Result<()> {
    let ffi_dep_sorter = DependencySorter::new(
        module.ffi_definitions.drain(..),
        FfiDefinitionDependencyLogic,
    );
    module.ffi_definitions = ffi_dep_sorter.sort();

    let type_sorter = DependencySorter::new(
        module.type_definitions.drain(..),
        TypeDefinitionDependencyLogic,
    );
    module.type_definitions = type_sorter.sort();
    Ok(())
}

// Generalized dependency sort using a version of depth-first topological sort:
//
// https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search
//
// Basically, we do a depth first search into the dependency graph, which ensures that we get dependencies
// first.
struct DependencySorter<L: DependencyLogic> {
    logic: L,
    unsorted: IndexMap<String, L::Item>,
    sorted: Vec<L::Item>,
}

impl<L: DependencyLogic> DependencySorter<L> {
    fn new(items: impl IntoIterator<Item = L::Item>, logic: L) -> Self {
        let unsorted: IndexMap<_, _> = items
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
        let Some(current_item) = self.unsorted.shift_remove(&current_name) else {
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
                .filter_map(|f| Self::type_dependency_name(&f.ty.ty))
                .collect(),
            FfiDefinition::RustFunction(func) => func
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func.return_type.ty)
                .filter_map(|ty| Self::type_dependency_name(&ty.ty))
                .collect(),
            FfiDefinition::FunctionType(func_type) => func_type
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func_type.return_type.ty)
                .filter_map(|ty| Self::type_dependency_name(&ty.ty))
                .collect(),
        }
    }
}

impl FfiDefinitionDependencyLogic {
    fn type_dependency_name(ffi_type: &FfiType) -> Option<String> {
        match &ffi_type {
            FfiType::Struct(name) => Some(name.0.clone()),
            FfiType::Function(name) => Some(name.0.clone()),
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
        match type_def {
            TypeDefinition::Simple(self_type)
            | TypeDefinition::Optional(OptionalType { self_type, .. })
            | TypeDefinition::Sequence(SequenceType { self_type, .. })
            | TypeDefinition::Map(MapType { self_type, .. })
            | TypeDefinition::Record(Record { self_type, .. })
            | TypeDefinition::Enum(Enum { self_type, .. })
            | TypeDefinition::Interface(Interface { self_type, .. })
            | TypeDefinition::CallbackInterface(CallbackInterface { self_type, .. })
            | TypeDefinition::Custom(CustomType { self_type, .. })
            | TypeDefinition::External(ExternalType { self_type, .. }) => {
                self_type.canonical_name.clone()
            }
        }
    }

    fn dependency_names(&self, type_def: &TypeDefinition) -> Vec<String> {
        match type_def {
            TypeDefinition::Simple(_) => vec![],
            TypeDefinition::Optional(OptionalType { inner, .. })
            | TypeDefinition::Sequence(SequenceType { inner, .. }) => {
                vec![inner.canonical_name.clone()]
            }
            TypeDefinition::Map(MapType { key, value, .. }) => {
                vec![key.canonical_name.clone(), value.canonical_name.clone()]
            }
            TypeDefinition::Record(r) => r
                .fields
                .iter()
                .map(|f| f.ty.canonical_name.clone())
                .collect(),
            TypeDefinition::Enum(e) => e
                .variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| f.ty.canonical_name.clone()))
                .collect(),
            TypeDefinition::Interface(i) => {
                i.trait_impls
                    .iter()
                    .map(|i| format!("Type{}", i.trait_name))
                    .chain(
                        i.methods
                            .iter()
                            .map(|meth| &meth.callable)
                            .chain(i.vtable.iter().flat_map(|vtable| {
                                vtable.methods.iter().map(|meth| &meth.callable)
                            }))
                            .flat_map(|callable| {
                                callable
                                    .arguments
                                    .iter()
                                    .map(|a| &a.ty)
                                    .chain(&callable.return_type.ty)
                                    .chain(&callable.throws_type.ty)
                                    .map(|ty| ty.canonical_name.clone())
                            }),
                    )
                    .collect()
            }
            TypeDefinition::CallbackInterface(c) => c
                .vtable
                .methods
                .iter()
                .flat_map(|m| {
                    m.callable
                        .arguments
                        .iter()
                        .map(|a| &a.ty)
                        .chain(&m.callable.return_type.ty)
                        .chain(&m.callable.throws_type.ty)
                })
                .map(|ty| ty.canonical_name.clone())
                .collect(),
            TypeDefinition::Custom(custom) => {
                vec![custom.builtin.canonical_name.clone()]
            }
            TypeDefinition::External(_) => vec![],
        }
    }
}
