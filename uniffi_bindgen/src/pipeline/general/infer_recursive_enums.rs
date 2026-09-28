/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::*;

/// Walk the structural containment graph of `type_defs` and mark every
/// `Enum` or `Record` that participates in a cycle as `recursive = true`.
///
/// Both enums and records are treated as nodes; edges are direct type
/// references in variant fields or record fields (unwrapping
/// `Optional`/`Sequence`/`Map` wrappers).
pub fn infer_recursive_types(type_defs: &mut [TypeDefinition]) {
    let deps = type_dep_graph(type_defs);
    let recursive = find_recursive_enum_names(&deps);
    mark_recursive_types(type_defs, &recursive);
}

/// Build the structural-containment graph for cycle detection.
///
/// Each node is either an enum name or a record name. Edges point from a
/// type to every other enum or record name directly reachable through its
/// fields (unwrapping `Optional`/`Sequence`/`Map` wrappers).
fn type_dep_graph(type_defs: &[TypeDefinition]) -> HashMap<String, HashSet<String>> {
    type_defs
        .iter()
        .filter_map(|td| match td {
            TypeDefinition::Enum(e) => {
                let deps: HashSet<String> = e
                    .variants
                    .iter()
                    .flat_map(|v| v.fields.iter())
                    .flat_map(|f| type_names_in_type(&f.ty.ty))
                    .collect();
                Some((e.name.clone(), deps))
            }
            TypeDefinition::Record(r) => {
                let deps: HashSet<String> = r
                    .fields
                    .iter()
                    .flat_map(|f| type_names_in_type(&f.ty.ty))
                    .collect();
                Some((r.name.clone(), deps))
            }
            _ => None,
        })
        .collect()
}

fn mark_recursive_types(type_defs: &mut [TypeDefinition], recursive: &HashSet<String>) {
    for td in type_defs.iter_mut() {
        match td {
            TypeDefinition::Enum(e) if recursive.contains(&e.name) => e.recursive = true,
            TypeDefinition::Record(r) if recursive.contains(&r.name) => r.recursive = true,
            _ => {}
        }
    }
}

/// Given a directed containment graph, return every node that participates in a cycle.
///
/// **Nodes** are type names (enums or records). **Edges** point from a type to
/// every other type it structurally contains, i.e. reachable through variant
/// or record fields, unwrapping `Optional`/`Sequence`/`Map` wrappers and custom types.
/// `Interface` references (`Arc<T>`) are treated as leaves.
///
/// `deps` maps each node to the nodes it directly contains (outgoing edges).
/// A node is in a cycle if any path of edges leads back to itself.
///
/// The cycles are found with Tarjan's strongly connected components algorithm.
/// A node is recursive if its component has more than one member, or if it has
/// an edge to itself. The result does not depend on the iteration order of `deps`.
///
/// This is the algorithmic core shared with `ComponentInterface::infer_recursive_types`.
pub(crate) fn find_recursive_enum_names<T>(deps: &HashMap<T, HashSet<T>>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    let mut state = TarjanState {
        deps,
        next_index: 0,
        index: HashMap::new(),
        lowlink: HashMap::new(),
        stack: Vec::new(),
        on_stack: HashSet::new(),
        recursive: HashSet::new(),
    };
    for name in deps.keys() {
        if !state.index.contains_key(name) {
            state.strong_connect(name);
        }
    }
    state.recursive
}

/// Bookkeeping for Tarjan's strongly connected components algorithm.
struct TarjanState<'a, T> {
    deps: &'a HashMap<T, HashSet<T>>,
    next_index: usize,
    /// Order in which each node was first visited.
    index: HashMap<&'a T, usize>,
    /// Smallest `index` reachable from each node while it is on `stack`.
    lowlink: HashMap<&'a T, usize>,
    stack: Vec<&'a T>,
    on_stack: HashSet<&'a T>,
    recursive: HashSet<T>,
}

impl<'a, T> TarjanState<'a, T>
where
    T: Eq + Hash + Clone,
{
    fn strong_connect(&mut self, node: &'a T) {
        let deps = self.deps;
        self.index.insert(node, self.next_index);
        self.lowlink.insert(node, self.next_index);
        self.next_index += 1;
        self.stack.push(node);
        self.on_stack.insert(node);

        let neighbors = deps.get(node);
        for dep in neighbors.into_iter().flatten() {
            if !self.index.contains_key(dep) {
                self.strong_connect(dep);
                let low = self.lowlink[node].min(self.lowlink[dep]);
                self.lowlink.insert(node, low);
            } else if self.on_stack.contains(dep) {
                let low = self.lowlink[node].min(self.index[dep]);
                self.lowlink.insert(node, low);
            }
        }

        // `node` is the root of a strongly connected component: pop the whole
        // component off the stack.
        if self.lowlink[node] == self.index[node] {
            let mut component = Vec::new();
            loop {
                let member = self
                    .stack
                    .pop()
                    .expect("the component root is still on the stack");
                self.on_stack.remove(member);
                component.push(member);
                if member == node {
                    break;
                }
            }
            let has_self_edge = neighbors.is_some_and(|n| n.contains(node));
            if component.len() > 1 || has_self_edge {
                self.recursive.extend(component.into_iter().cloned());
            }
        }
    }
}

/// Return all enum and record names directly reachable from `ty`.
///
/// Unwraps `Optional`/`Sequence`/`Map` wrappers but does not cross type
/// definition boundaries — both `Enum` and `Record` references are returned
/// as-is rather than recursed into, because they are nodes in their own right.
///
/// Custom types are followed through to their builtin type.
fn type_names_in_type(ty: &Type) -> Vec<String> {
    match ty {
        Type::Enum { name, .. } | Type::Record { name, .. } => vec![name.clone()],
        Type::Custom { builtin, .. } => type_names_in_type(builtin),
        Type::Box { inner_type }
        | Type::Optional { inner_type }
        | Type::Sequence { inner_type }
        | Type::Set { inner_type } => type_names_in_type(inner_type),
        Type::Map {
            key_type,
            value_type,
        } => {
            let mut names = type_names_in_type(key_type);
            names.extend(type_names_in_type(value_type));
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

    #[test]
    fn find_recursive_overlapping_cycles_is_order_independent() {
        // T -> U -> T and T -> V -> U -> T overlap, so all three nodes are in a cycle.
        // Every new `HashMap` gets its own hash seed, so building the graph many times
        // covers the different iteration orders.
        for _ in 0..64 {
            let deps = make_graph(&[("T", &["U", "V"]), ("U", &["T"]), ("V", &["U"])]);
            assert_eq!(
                find_recursive_enum_names(&deps),
                HashSet::from(["T", "U", "V"])
            );
        }
    }

    fn make_type_node(ty: Type) -> TypeNode {
        TypeNode {
            id: 0, // not used for these tests
            canonical_name: "Test".to_string(),
            is_used_as_error: false,
            has_from_unexpected_callback_error_impl: false,
            ffi_type: FfiType::RustBuffer(None),
            ty,
        }
    }

    fn make_enum_with_field_type(name: &str, field_ty: Type) -> TypeDefinition {
        let discr_type_node = make_type_node(Type::Int8);
        let self_type = make_type_node(Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
            orig_name: name.to_string(),
        });
        TypeDefinition::Enum(Enum {
            module_path: "my_crate".into(),
            name: name.to_string(),
            orig_name: name.to_string(),
            is_flat: false,
            self_type,
            discr_specified: false,
            discr_type: discr_type_node.clone(),
            uniffi_trait_methods: UniffiTraitMethods::default(),
            shape: EnumShape::Enum,
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            variants: vec![Variant {
                name: "Variant0".to_string(),
                orig_name: "Variant0".to_string(),
                discr: Literal::Int(0, Radix::Decimal, discr_type_node),
                fields_kind: FieldsKind::Unnamed,
                fields: vec![Field {
                    name: "value".to_string(),
                    orig_name: "value".to_string(),
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
            orig_name: name.to_string(),
        });
        TypeDefinition::Enum(Enum {
            module_path: "my_crate".into(),
            name: name.to_string(),
            orig_name: name.to_string(),
            is_flat: true,
            self_type,
            discr_specified: false,
            discr_type: discr_type_node.clone(),
            uniffi_trait_methods: UniffiTraitMethods::default(),
            shape: EnumShape::Enum,
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            variants: vec![Variant {
                name: "Unit".to_string(),
                orig_name: "Unit".to_string(),
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
            orig_name: name.to_string(),
        }
    }

    fn record_type(name: &str) -> Type {
        Type::Record {
            namespace: "test".to_string(),
            name: name.to_string(),
            orig_name: name.to_string(),
        }
    }

    fn custom_type(name: &str, builtin: Type) -> Type {
        Type::Custom {
            namespace: "test".to_string(),
            name: name.to_string(),
            orig_name: name.to_string(),
            builtin: Box::new(builtin),
        }
    }

    // Like `make_enum_with_field_type` but one variant per entry in `field_types`.
    fn make_enum_with_variants(name: &str, field_types: Vec<Type>) -> TypeDefinition {
        let discr_type_node = make_type_node(Type::Int8);
        let self_type = make_type_node(Type::Enum {
            namespace: "test".to_string(),
            name: name.to_string(),
            orig_name: name.to_string(),
        });
        let variants = field_types
            .into_iter()
            .enumerate()
            .map(|(i, field_ty)| Variant {
                name: format!("Variant{i}"),
                orig_name: format!("Variant{i}"),
                discr: Literal::Int(i as i64, Radix::Decimal, discr_type_node.clone()),
                fields_kind: FieldsKind::Unnamed,
                fields: vec![Field {
                    name: "value".to_string(),
                    orig_name: "value".to_string(),
                    ty: make_type_node(field_ty),
                    default: None,
                    docstring: None,
                }],
                docstring: None,
            })
            .collect();
        TypeDefinition::Enum(Enum {
            module_path: "my_crate".into(),
            name: name.to_string(),
            orig_name: name.to_string(),
            is_flat: false,
            self_type,
            discr_specified: false,
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

    fn make_record_with_field_type(name: &str, field_ty: Type) -> TypeDefinition {
        let self_type = make_type_node(Type::Record {
            namespace: "test".to_string(),
            name: name.to_string(),
            orig_name: name.to_string(),
        });
        TypeDefinition::Record(Record {
            module_path: "my_crate".into(),
            name: name.to_string(),
            orig_name: name.to_string(),
            self_type,
            fields_kind: FieldsKind::Named,
            uniffi_trait_methods: UniffiTraitMethods::default(),
            constructors: vec![],
            methods: vec![],
            docstring: None,
            recursive: false,
            fields: vec![Field {
                name: "value".to_string(),
                orig_name: "value".to_string(),
                ty: make_type_node(field_ty),
                default: None,
                docstring: None,
            }],
        })
    }

    #[test]
    fn dep_graph_no_deps() {
        let defs = vec![make_flat_enum("Apple"), make_flat_enum("Orange")];
        let graph = type_dep_graph(&defs);
        assert!(graph["Apple"].is_empty());
        assert!(graph["Orange"].is_empty());
    }

    #[test]
    fn dep_graph_self_referential() {
        let defs = vec![make_enum_with_field_type("Quine", enum_type("Quine"))];
        let graph = type_dep_graph(&defs);
        assert_eq!(graph["Quine"], HashSet::from(["Quine".to_string()]));
    }

    #[test]
    fn dep_graph_deduplicates_repeated_dep() {
        // Socks has two variants both containing Sock — Sock should appear once in the dep set.
        let defs = vec![
            make_enum_with_variants("Socks", vec![enum_type("Sock"), enum_type("Sock")]),
            make_flat_enum("Sock"),
        ];
        let graph = type_dep_graph(&defs);
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
        let graph = type_dep_graph(&defs);
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
        let graph = type_dep_graph(&defs);
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
        let graph = type_dep_graph(&defs);
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
                    orig_name: "Judge".to_string(),
                    imp: ObjectImpl::Struct,
                },
            ),
            make_flat_enum("Sentence"),
        ];
        let graph = type_dep_graph(&defs);
        assert!(graph["Verdict"].is_empty());
    }

    #[test]
    fn dep_graph_enum_record_cycle() {
        // RoseTree → RoseData (via Branch field), RoseData → RoseTree (via Vec field).
        let defs = vec![
            make_enum_with_field_type("RoseTree", record_type("RoseData")),
            make_record_with_field_type(
                "RoseData",
                Type::Sequence {
                    inner_type: Box::new(enum_type("RoseTree")),
                },
            ),
        ];
        let graph = type_dep_graph(&defs);
        assert_eq!(graph["RoseTree"], HashSet::from(["RoseData".to_string()]));
        assert_eq!(graph["RoseData"], HashSet::from(["RoseTree".to_string()]));

        let recursive = find_recursive_enum_names(&graph);
        assert_eq!(
            recursive,
            HashSet::from(["RoseTree".to_string(), "RoseData".to_string()])
        );
    }

    #[test]
    fn dep_graph_custom_type_propagates_dep() {
        // Wrapped is a custom type whose builtin is the Inner record.
        let defs = vec![
            make_enum_with_field_type("Outer", custom_type("Wrapped", record_type("Inner"))),
            make_record_with_field_type("Inner", Type::String),
        ];
        let graph = type_dep_graph(&defs);
        assert_eq!(graph["Outer"], HashSet::from(["Inner".to_string()]));
    }

    #[test]
    fn infer_recursive_through_custom_type() {
        // Node { W(Wrapped), Leaf }, where Wrapped is a custom type over
        // Inner { next: Option<Box<Node>> }.
        let mut node =
            make_enum_with_field_type("Node", custom_type("Wrapped", record_type("Inner")));
        if let TypeDefinition::Enum(e) = &mut node {
            e.variants.push(Variant {
                name: "Leaf".to_string(),
                orig_name: "Leaf".to_string(),
                discr: Literal::Int(1, Radix::Decimal, make_type_node(Type::Int8)),
                fields_kind: FieldsKind::Unit,
                fields: vec![],
                docstring: None,
            });
        }
        let wrapped = TypeDefinition::Custom(CustomType {
            self_type: make_type_node(custom_type("Wrapped", record_type("Inner"))),
            module_path: "my_crate".into(),
            orig_name: "Wrapped".to_string(),
            name: "Wrapped".to_string(),
            builtin: make_type_node(record_type("Inner")),
            docstring: None,
        });
        let inner = make_record_with_field_type(
            "Inner",
            Type::Optional {
                inner_type: Box::new(Type::Box {
                    inner_type: Box::new(enum_type("Node")),
                }),
            },
        );
        let mut defs = vec![node, wrapped, inner];
        infer_recursive_types(&mut defs);

        let recursive: HashSet<&str> = defs
            .iter()
            .filter_map(|td| match td {
                TypeDefinition::Enum(e) if e.recursive => Some(e.name.as_str()),
                TypeDefinition::Record(r) if r.recursive => Some(r.name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(recursive, HashSet::from(["Node", "Inner"]));
    }
}
