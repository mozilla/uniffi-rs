/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::*;

/// Walk the structural containment graph of `type_defs` and mark every
/// `Enum` that participates in a cycle as `recursive = true`.
pub fn infer_recursive_enums(type_defs: &mut [TypeDefinition]) {
    let deps = enum_dep_graph(type_defs);
    let recursive = find_recursive_enum_names(&deps);
    mark_recursive_enums(type_defs, &recursive);
}

/// Build the structural-containment graph for `find_recursive_enum_names`.
///
/// Each key is an enum name; its value is the set of enum names reachable
/// through its variant fields (unwrapping `Optional`/`Sequence`/`Map` wrappers
/// and crossing `Record` boundaries).
fn enum_dep_graph(type_defs: &[TypeDefinition]) -> HashMap<String, HashSet<String>> {
    type_defs
        .iter()
        .filter_map(|td| {
            if let TypeDefinition::Enum(e) = td {
                Some(e)
            } else {
                None
            }
        })
        .map(|e| {
            let deps: HashSet<String> = e
                .variants
                .iter()
                .flat_map(|v| v.fields.iter())
                .flat_map(|f| enum_names_in_type(&f.ty.ty, type_defs))
                .collect();
            (e.name.clone(), deps)
        })
        .collect()
}

fn mark_recursive_enums(type_defs: &mut [TypeDefinition], recursive: &HashSet<String>) {
    for td in type_defs.iter_mut() {
        if let TypeDefinition::Enum(e) = td {
            if recursive.contains(&e.name) {
                e.recursive = true;
            }
        }
    }
}

/// Given a directed containment graph, return every node that participates in a cycle.
///
/// **Nodes** are enum names. **Edges** point from an enum to every other enum
/// it structurally contains — i.e. reachable through variant fields, unwrapping
/// `Optional`/`Sequence`/`Map` wrappers, and crossing `Record` field boundaries.
/// `Interface` references (`Arc<T>`) are treated as leaves: following method
/// signatures would produce false positives.
///
/// `deps` maps each node to the nodes it directly contains (outgoing edges).
/// A node is in a cycle if any path of edges leads back to itself.
///
/// This is the algorithmic core shared with `ComponentInterface::infer_recursive_enums`.
pub(crate) fn find_recursive_enum_names<T>(deps: &HashMap<T, HashSet<T>>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    let mut recursive = HashSet::new();
    let mut visited = HashSet::new();
    for name in deps.keys() {
        let mut stack = Vec::new();
        dfs(name, deps, &mut stack, &mut visited, &mut recursive);
    }
    recursive
}

fn dfs<T>(
    name: &T,
    deps: &HashMap<T, HashSet<T>>,
    stack: &mut Vec<T>,
    visited: &mut HashSet<T>,
    recursive: &mut HashSet<T>,
) where
    T: Eq + Hash + Clone,
{
    // A visited node was fully explored in a prior DFS tree. Any cycle passing
    // through it would have been detected during that traversal and its members
    // added to `recursive`. Safe to skip.
    if visited.contains(name) {
        return;
    }
    if let Some(pos) = stack.iter().position(|s| s == name) {
        // Back edge: every node from pos..end is in a cycle
        for cycle_member in &stack[pos..] {
            recursive.insert(cycle_member.clone());
        }
        return;
    }
    stack.push(name.clone());
    if let Some(enum_deps) = deps.get(name) {
        for dep in enum_deps {
            dfs(dep, deps, stack, visited, recursive);
        }
    }
    stack.pop();
    visited.insert(name.clone());
}

fn enum_names_in_type(ty: &Type, type_defs: &[TypeDefinition]) -> Vec<String> {
    enum_names_in_type_inner(ty, type_defs, &mut HashSet::new())
}

fn enum_names_in_type_inner(
    ty: &Type,
    type_defs: &[TypeDefinition],
    visited_records: &mut HashSet<String>,
) -> Vec<String> {
    match ty {
        Type::Enum { name, .. } => vec![name.clone()],
        Type::Optional { inner_type } | Type::Sequence { inner_type } => {
            enum_names_in_type_inner(inner_type, type_defs, visited_records)
        }
        Type::Map {
            key_type,
            value_type,
        } => {
            enum_names_in_type_inner(key_type, type_defs, visited_records);
            enum_names_in_type_inner(value_type, type_defs, visited_records)
        }
        Type::Record { name, .. } => {
            // Guard against mutually-recursive records (e.g. RecordA containing
            // Option<RecordB> and RecordB containing Option<RecordA>), which would
            // otherwise cause infinite recursion.
            if !visited_records.insert(name.clone()) {
                return vec![];
            }
            let record = type_defs.iter().find_map(|td| {
                if let TypeDefinition::Record(r) = td {
                    if &r.name == name {
                        Some(r)
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
            let Some(record) = record else {
                return vec![];
            };
            let mut names = Vec::new();
            for f in &record.fields {
                names.extend(enum_names_in_type_inner(
                    &f.ty.ty,
                    type_defs,
                    visited_records,
                ));
            }
            names
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph<'a>(pairs: &[(&'a str, &[&'a str])]) -> HashMap<&'a str, HashSet<&'a str>> {
        pairs
            .iter()
            .map(|&(k, vs)| (k, vs.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn find_recursive_self_cycle() {
        let deps = make_graph(&[("Quine", &["Quine"])]);
        assert_eq!(find_recursive_enum_names(&deps), HashSet::from(["Quine"]));
    }

    #[test]
    fn find_recursive_mutual_cycle() {
        let deps = make_graph(&[("Chicken", &["Egg"]), ("Egg", &["Chicken"])]);
        assert_eq!(
            find_recursive_enum_names(&deps),
            HashSet::from(["Chicken", "Egg"])
        );
    }

    #[test]
    fn find_recursive_longer_cycle() {
        // All three form a cycle.
        let deps = make_graph(&[
            ("Rock", &["Paper"]),
            ("Paper", &["Scissors"]),
            ("Scissors", &["Rock"]),
        ]);
        assert_eq!(
            find_recursive_enum_names(&deps),
            HashSet::from(["Rock", "Paper", "Scissors"])
        );
    }

    #[test]
    fn find_recursive_chain_no_cycle() {
        let deps = make_graph(&[("Morning", &["Noon"]), ("Noon", &["Night"]), ("Night", &[])]);
        assert!(find_recursive_enum_names(&deps).is_empty());
    }

    #[test]
    fn find_recursive_partial_cycle() {
        // Inspector points into the Suspect/Accomplice cycle but is not part of it.
        let deps = make_graph(&[
            ("Inspector", &["Suspect"]),
            ("Suspect", &["Accomplice"]),
            ("Accomplice", &["Suspect"]),
        ]);
        assert_eq!(
            find_recursive_enum_names(&deps),
            HashSet::from(["Suspect", "Accomplice"])
        );
    }

    fn make_type_node(ty: Type) -> TypeNode {
        TypeNode {
            canonical_name: "Test".to_string(),
            is_used_as_error: false,
            ffi_type: FfiType::RustBuffer(None),
            ty,
        }
    }

    fn make_enum_with_field_type(name: &str, field_ty: Type) -> TypeDefinition {
        let discr_type_node = make_type_node(Type::Int8);
        let self_type = make_type_node(Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
        });
        TypeDefinition::Enum(Enum {
            name: name.to_string(),
            is_flat: false,
            self_type,
            discr_type: discr_type_node.clone(),
            uniffi_trait_methods: UniffiTraitMethods::default(),
            shape: EnumShape::Enum,
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            variants: vec![Variant {
                name: "Variant0".to_string(),
                discr: Literal::Int(0, Radix::Decimal, discr_type_node),
                fields_kind: FieldsKind::Unnamed,
                fields: vec![Field {
                    name: "value".to_string(),
                    ty: make_type_node(field_ty),
                    default: None,
                    docstring: None,
                }],
                docstring: None,
            }],
        })
    }

    fn make_flat_enum(name: &str) -> TypeDefinition {
        let discr_type_node = make_type_node(Type::Int8);
        let self_type = make_type_node(Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
        });
        TypeDefinition::Enum(Enum {
            name: name.to_string(),
            is_flat: true,
            self_type,
            discr_type: discr_type_node.clone(),
            uniffi_trait_methods: UniffiTraitMethods::default(),
            shape: EnumShape::Enum,
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            variants: vec![Variant {
                name: "Unit".to_string(),
                discr: Literal::Int(0, Radix::Decimal, discr_type_node),
                fields_kind: FieldsKind::Unit,
                fields: vec![],
                docstring: None,
            }],
        })
    }

    fn enum_type(name: &str) -> Type {
        Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
        }
    }

    // Like `make_enum_with_field_type` but one variant per entry in `field_types`.
    fn make_enum_with_variants(name: &str, field_types: Vec<Type>) -> TypeDefinition {
        let discr_type_node = make_type_node(Type::Int8);
        let self_type = make_type_node(Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
        });
        let variants = field_types
            .into_iter()
            .enumerate()
            .map(|(i, field_ty)| Variant {
                name: format!("Variant{i}"),
                discr: Literal::Int(i as i64, Radix::Decimal, discr_type_node.clone()),
                fields_kind: FieldsKind::Unnamed,
                fields: vec![Field {
                    name: "value".to_string(),
                    ty: make_type_node(field_ty),
                    default: None,
                    docstring: None,
                }],
                docstring: None,
            })
            .collect();
        TypeDefinition::Enum(Enum {
            name: name.to_string(),
            is_flat: false,
            self_type,
            discr_type: discr_type_node,
            uniffi_trait_methods: UniffiTraitMethods::default(),
            shape: EnumShape::Enum,
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            variants,
        })
    }

    #[test]
    fn dep_graph_no_deps() {
        let defs = vec![make_flat_enum("Apple"), make_flat_enum("Orange")];
        let graph = enum_dep_graph(&defs);
        assert!(graph["Apple"].is_empty());
        assert!(graph["Orange"].is_empty());
    }

    #[test]
    fn dep_graph_self_referential() {
        let defs = vec![make_enum_with_field_type("Quine", enum_type("Quine"))];
        let graph = enum_dep_graph(&defs);
        assert_eq!(graph["Quine"], HashSet::from(["Quine".to_string()]));
    }

    #[test]
    fn dep_graph_deduplicates_repeated_dep() {
        // Socks has two variants both containing Sock — Sock should appear once in the dep set.
        let defs = vec![
            make_enum_with_variants("Socks", vec![enum_type("Sock"), enum_type("Sock")]),
            make_flat_enum("Sock"),
        ];
        let graph = enum_dep_graph(&defs);
        assert_eq!(graph["Socks"], HashSet::from(["Sock".to_string()]));
    }

    #[test]
    fn dep_graph_optional_field_propagates_dep() {
        let defs = vec![
            make_enum_with_field_type(
                "Outer",
                Type::Optional {
                    inner_type: Box::new(enum_type("Inner")),
                },
            ),
            make_flat_enum("Inner"),
        ];
        let graph = enum_dep_graph(&defs);
        assert_eq!(graph["Outer"], HashSet::from(["Inner".to_string()]));
    }

    #[test]
    fn dep_graph_sequence_field_propagates_dep() {
        let defs = vec![
            make_enum_with_field_type(
                "Outer",
                Type::Sequence {
                    inner_type: Box::new(enum_type("Inner")),
                },
            ),
            make_flat_enum("Inner"),
        ];
        let graph = enum_dep_graph(&defs);
        assert_eq!(graph["Outer"], HashSet::from(["Inner".to_string()]));
    }

    #[test]
    fn dep_graph_map_value_propagates_dep() {
        let defs = vec![
            make_enum_with_field_type(
                "Outer",
                Type::Map {
                    key_type: Box::new(Type::String),
                    value_type: Box::new(enum_type("Inner")),
                },
            ),
            make_flat_enum("Inner"),
        ];
        let graph = enum_dep_graph(&defs);
        assert_eq!(graph["Outer"], HashSet::from(["Inner".to_string()]));
    }

    #[test]
    fn dep_graph_interface_field_has_no_dep() {
        // Verdict holds a Judge (Interface). Even though Sentence exists in the type universe,
        // Verdict must not gain it as a dep — the traversal stops at Interface boundaries.
        let defs = vec![
            make_enum_with_field_type(
                "Verdict",
                Type::Interface {
                    namespace: "test".to_string(),
                    name: "Judge".to_string(),
                    imp: ObjectImpl::Struct,
                },
            ),
            make_flat_enum("Sentence"),
        ];
        let graph = enum_dep_graph(&defs);
        assert!(graph["Verdict"].is_empty());
    }
}
