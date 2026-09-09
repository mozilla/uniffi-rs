/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Sort definitions so that dependencies come first
//!
//! This is needed for languages like Python that will throw errors if the dependent type is
//! defined by its dependency.

use super::*;

pub fn sort_type_definitions(
    type_definitions: impl IntoIterator<Item = TypeDefinition>,
) -> Vec<TypeDefinition> {
    let type_sorter = DependencySorter::new(type_definitions, TypeDefinitionDependencyLogic);
    type_sorter.sort()
}

pub fn sort_ffi_definitions(
    ffi_definitions: impl IntoIterator<Item = FfiDefinition>,
) -> Vec<FfiDefinition> {
    let ffi_dep_sorter = DependencySorter::new(ffi_definitions, FfiDefinitionDependencyLogic);
    ffi_dep_sorter.sort()
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
                .filter_map(|f| Self::type_dependency_name(&f.ty))
                .collect(),
            FfiDefinition::RustFunction(func) => func
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func.return_type.ty)
                .filter_map(Self::type_dependency_name)
                .collect(),
            FfiDefinition::FunctionType(func_type) => func_type
                .arguments
                .iter()
                .map(|a| &a.ty)
                .chain(&func_type.return_type.ty)
                .filter_map(Self::type_dependency_name)
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

impl TypeDefinitionDependencyLogic {
    fn type_name(ty: &TypeNode) -> String {
        match ty.ty.namespace() {
            Some(namespace) => format!("{namespace}.{}", ty.canonical_name),
            None => ty.canonical_name.clone(),
        }
    }
}

impl DependencyLogic for TypeDefinitionDependencyLogic {
    type Item = TypeDefinition;

    fn item_name(&self, type_def: &TypeDefinition) -> String {
        match type_def {
            TypeDefinition::Simple(self_type)
            | TypeDefinition::Box(BoxedType { self_type, .. })
            | TypeDefinition::Optional(OptionalType { self_type, .. })
            | TypeDefinition::Sequence(SequenceType { self_type, .. })
            | TypeDefinition::Map(MapType { self_type, .. })
            | TypeDefinition::Set(SetType { self_type, .. })
            | TypeDefinition::Record(Record { self_type, .. })
            | TypeDefinition::Enum(Enum { self_type, .. })
            | TypeDefinition::Interface(Interface { self_type, .. })
            | TypeDefinition::CallbackInterface(CallbackInterface { self_type, .. })
            | TypeDefinition::Custom(CustomType { self_type, .. }) => Self::type_name(self_type),
            TypeDefinition::External(ExternalType { self_type, .. }) => {
                format!("External:{}", Self::type_name(self_type))
            }
        }
    }

    fn dependency_names(&self, type_def: &TypeDefinition) -> Vec<String> {
        match type_def {
            TypeDefinition::Simple(_) => vec![],
            TypeDefinition::Box(BoxedType { inner, .. })
            | TypeDefinition::Optional(OptionalType { inner, .. })
            | TypeDefinition::Sequence(SequenceType { inner, .. })
            | TypeDefinition::Set(SetType { inner, .. }) => {
                vec![Self::type_name(inner)]
            }
            TypeDefinition::Map(MapType { key, value, .. }) => {
                vec![Self::type_name(key), Self::type_name(value)]
            }
            TypeDefinition::Record(r) => r.fields.iter().map(|f| Self::type_name(&f.ty)).collect(),
            TypeDefinition::Enum(e) => e
                .variants
                .iter()
                .flat_map(|v| v.fields.iter().map(|f| Self::type_name(&f.ty)))
                .collect(),
            TypeDefinition::Interface(i) => {
                i.trait_impls
                    .iter()
                    .map(|i| Self::type_name(&i.trait_ty))
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
                                    .map(Self::type_name)
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
                .map(Self::type_name)
                .collect(),
            TypeDefinition::Custom(custom) => {
                vec![Self::type_name(&custom.builtin)]
            }
            TypeDefinition::External(_) => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom_definition(namespace: &str, name: &str) -> TypeDefinition {
        TypeDefinition::Custom(CustomType {
            self_type: TypeNode {
                // How `types::canonical_name` names a user type
                canonical_name: format!("Type{name}"),
                id: 0,
                is_used_as_error: false,
                has_from_unexpected_callback_error_impl: false,
                ffi_type: FfiType::UInt64,
                ty: Type::Custom {
                    namespace: namespace.to_string(),
                    name: name.to_string(),
                    orig_name: name.to_string(),
                    builtin: Box::new(Type::UInt64),
                },
            },
            module_path: namespace.to_string(),
            orig_name: name.to_string(),
            name: name.to_string(),
            builtin: TypeNode {
                canonical_name: "UInt64".to_string(),
                id: 1,
                is_used_as_error: false,
                has_from_unexpected_callback_error_impl: false,
                ffi_type: FfiType::UInt64,
                ty: Type::UInt64,
            },
            docstring: None,
        })
    }

    /// The `External` entry a namespace gets for a type defined in another one
    ///
    /// Built the way `type_definitions_from_api` builds it: from the type's own self
    /// type, recording the namespace that *defines* the type rather than the one using
    /// it.
    fn external_for(type_def: &TypeDefinition) -> TypeDefinition {
        let TypeDefinition::Custom(custom) = type_def else {
            panic!("expected a custom type definition, got {type_def:?}");
        };
        let self_type = custom.self_type.clone();
        TypeDefinition::External(ExternalType {
            namespace: self_type
                .ty
                .namespace()
                .expect("a user type has a namespace")
                .to_string(),
            name: custom.name.clone(),
            self_type,
        })
    }

    #[test]
    fn test_same_named_types_in_different_namespaces_are_both_kept() {
        let sorted = sort_type_definitions(vec![
            custom_definition("crate_a", "Thing"),
            custom_definition("crate_b", "Thing"),
        ]);
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn test_external_types_do_not_replace_real_definitions() {
        let custom = &custom_definition("crate_a", "Thing");
        let external = &external_for(custom);
        for type_definitions in [[custom, external], [external, custom]] {
            let sorted = sort_type_definitions(type_definitions.into_iter().cloned());
            assert_eq!(sorted.len(), 2, "a definition was dropped: {sorted:?}");
        }
    }
}
