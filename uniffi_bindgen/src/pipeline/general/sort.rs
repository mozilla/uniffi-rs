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
    // Multiple items can share a name: when sorting definitions across namespaces, a type
    // used outside its defining namespace has both its real definition and an
    // `External` marker under the same canonical name.  Keep all of them — a map keyed by
    // name alone would silently drop definitions.
    unsorted: IndexMap<String, Vec<L::Item>>,
    sorted: Vec<L::Item>,
}

impl<L: DependencyLogic> DependencySorter<L> {
    fn new(items: impl IntoIterator<Item = L::Item>, logic: L) -> Self {
        let mut unsorted: IndexMap<String, Vec<L::Item>> = IndexMap::new();
        for i in items {
            unsorted.entry(logic.item_name(&i)).or_default().push(i);
        }
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
        let Some(current_items) = self.unsorted.shift_remove(&current_name) else {
            // If `current_name` is not in unsorted, then we've already processed the item
            return;
        };
        // Add all dependents first
        let dependency_names: Vec<String> = current_items
            .iter()
            .flat_map(|item| self.logic.dependency_names(item))
            .collect();
        for name in dependency_names {
            self.recurse(name);
        }
        // Then add the current items
        self.sorted.extend(current_items);
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
            | TypeDefinition::Custom(CustomType { self_type, .. })
            | TypeDefinition::External(ExternalType { self_type, .. }) => {
                self_type.canonical_name.clone()
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
                    .map(|i| i.trait_ty.canonical_name.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test item: a name, a marker to tell same-named items apart, and dependency names
    struct TestItem {
        name: &'static str,
        marker: &'static str,
        deps: Vec<&'static str>,
    }

    struct TestLogic;

    impl DependencyLogic for TestLogic {
        type Item = TestItem;

        fn item_name(&self, item: &TestItem) -> String {
            item.name.to_string()
        }

        fn dependency_names(&self, item: &TestItem) -> Vec<String> {
            item.deps.iter().map(|s| s.to_string()).collect()
        }
    }

    fn sort(items: Vec<TestItem>) -> Vec<&'static str> {
        DependencySorter::new(items, TestLogic)
            .sort()
            .into_iter()
            .map(|i| i.marker)
            .collect()
    }

    #[test]
    fn test_dependencies_sort_before_dependents() {
        let order = sort(vec![
            TestItem {
                name: "TypeRecord",
                marker: "record",
                deps: vec!["TypeInner", "String"],
            },
            TestItem {
                name: "String",
                marker: "string",
                deps: vec![],
            },
            TestItem {
                name: "TypeInner",
                marker: "inner",
                deps: vec!["String"],
            },
        ]);
        assert_eq!(order, vec!["string", "inner", "record"]);
    }

    #[test]
    fn test_same_named_items_are_all_kept() {
        // Multiple definitions can share a name: a type used outside its defining
        // namespace has both its real definition and an `External` marker under the
        // same canonical name (e.g. `TypeDefinition::Custom` in the defining namespace
        // and `TypeDefinition::External` in the using one).  The sorter must keep all
        // of them — dropping the real definition breaks consumers that need it, like
        // the FFI type oracles.
        let order = sort(vec![
            TestItem {
                name: "TypeRecord",
                marker: "record",
                deps: vec!["TypeCounter"],
            },
            TestItem {
                name: "TypeCounter",
                marker: "external",
                deps: vec![],
            },
            TestItem {
                name: "TypeCounter",
                marker: "custom",
                deps: vec!["String"],
            },
            TestItem {
                name: "String",
                marker: "string",
                deps: vec![],
            },
        ]);

        // All four definitions survive
        assert_eq!(order.len(), 4);
        let pos = |marker| order.iter().position(|m| *m == marker).unwrap();
        // Both same-named definitions are present ...
        assert!(order.contains(&"external"));
        assert!(order.contains(&"custom"));
        // ... and dependencies still come before dependents
        assert!(pos("string") < pos("custom"));
        assert!(pos("custom") < pos("record"));
    }

    #[test]
    fn test_dependency_cycles_terminate() {
        let order = sort(vec![
            TestItem {
                name: "TypeA",
                marker: "a",
                deps: vec!["TypeB"],
            },
            TestItem {
                name: "TypeB",
                marker: "b",
                deps: vec!["TypeA"],
            },
        ]);
        assert_eq!(order.len(), 2);
    }
}
