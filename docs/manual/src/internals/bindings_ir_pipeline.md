# Bindings IR Passes

Bindings IR passes are used to transform different [intermediate representations of the generated bindings](./bindings_ir.md).

* `uniffi_pipeline` defines the foundational code
* `uniffi_bindgen` defines the general pipeline -- this converts `uniffi_meta` metadata to the general IR
* Each language defines their own pipeline, which extends the general pipeline to output their language-specific IR.

## Nodes

Types inside an are are called "nodes" and they must derive a `Node` trait impl.
The top-level node is always named `Root`.
When this doc talks about converting between two IRs, in concrete terms this means converting
between the root nodes of those IRs.

`Node` trait provides functionality to traverse the tree of nodes.
`Node::visit()` and `Node::visit_mut()` allows you to visit all descendants with a given type.

## Node conversions

`Node` also provides functionality to convert between different IR types.
`Node::try_from_node` converts a node from one IR to the corresponding node from another IR.
For example, `from_ir::Node` will be converted to `into_ir::Node` using the following rules:

* Any fields in `into_ir::Node`, but not `from_ir::Node` are added as empty values (similar to `Default::default`).
* Any fields in `from_ir::Node`, but not `into_ir::Node` are ignored
* Any fields in both are recursively converted using the same process.

Attributes can be used to customize these rules.

Renamed node types, variants, and fields can be handled by adding the `#[node(from([name]))]` attribute:

```rust
#[derive(Node)]
pub struct Node {
    /// `from_ir::Node::prev_name` fields become `into_ir::Node::new_name` fields.
    #[node(from(prev_name))]
    new_name: String,
}
```

Also, nodes can be "wrapped" in the new IR using the `#[node(wraps([type]))]` attribute:

```rust
#[derive(Node)]
pub struct Node {
    /// `into_ir::Node` wraps `from_ir::WrappedNode`.
    /// All `from_ir::WrappedNode` values will be replaced with `Node { wrapped: wrapped_node }` values.
    /// Any other fields will be initialized to their empty values.
    #[node(wraps)]
    wrapped: WrappedNode,
}

```

## Pipeline passes

Suppose you want to convert nodes in the `from_ir` module to nodes in the `into_ir` module.
The typical pattern for this is:

* Define a `into_ir::pass` module, which contains nodes that are the union of `from_ir` and `into_ir`.
  These nodes will contain fields from both `from_ir` and `into_ir`
* Use `Pipeline::convert_ir_pass` to convert the `from_ir` nodes to `into_ir::pass` nodes.
* Use `Pipeline::pass` add multiple passes that mutate the `into_ir::pass` nodes.
  These passes will normally move/copy date from the old fields into the new fields.
* Use `Pipeline::convert_ir_pass` to convert the `into_ir::pass` to `into_ir` nodes.

For example, one thing the initial IR -> general IR pass does is convert/normalize module paths into module names.
This could be accomplished something like this:

```rust
mod initial {
    use uniffi_pipeline::Node;

    // Types in the initial IR have a `module_path` field
    //
    // There's more variants and fields in the actual definition, but we'll
    // just focus on module_path/module_name
    #[derive(Node)]
    pub enum Type {
        Record {
            module_path: String,
        },
        Enum {
            module_path: String,
        },
        Interface {
            module_path: String,
        },
    }
}

mod general {
    use uniffi_pipeline::Node;

    // Types in the general IR have a `module_name` field
    #[derive(Node)]
    pub enum Type {
        Record {
            module_name: String,
        },
        Enum {
            module_name: String,
        },
        Interface {
            module_name: String,
        },
    }

    mod pass {
        use crate::pipeline::{initial, general};
        use uniffi_pipeline::{new_pipeline, Node, Pipeline};
        // Types in the general::pass IR have both `module_path` and `module_name`
        #[derive(Node)]
        pub enum Type {
            Record {
                module_path: String,
                module_name: String,
            },
            Enum {
                module_path: String,
                module_name: String,
            },
            Interface {
                module_path: String,
                module_name: String,
            },
        }

        pub fn pipeline() -> Pipeline<initial::Root, general::Root> {
            new_pipeline()
                .convert_ir_pass::<Root>()
                .pass(normalize_module_names)
                // Normally there would be many more passes here and passes
                // would be in separate modules.
                .convert_ir_pass::<general::Root>()
        }

        pub fn normalize_module_names(root: &mut pass::Type) -> Result<()> {
            match ty {
                Type::Record { module_name, module_path }
                | Type::Enum { module_name, module_path }
                | Type::Inteface { module_name, module_path } => {
                    // normalize module_path -> module_name
                    //
                    // The `take()` method is used to remove the string data from `module_path` leaving
                    // behind an empty node.  This optimization avoids cloning `module_path`, which we're
                    // going to throw away at the end of the pass anyway.
                    *module_name = normalize_module_path(module_path.take());
                }
            }
            Ok(())
        }
    }
}
```

This is why the `pass` IR contains fields from both the `from` and `to` IRs:
the pass can access both the old fields and new fields while transforming the data.

Note: Module name normalization is actually done as part of a larger step that moves modules into a tree.
This example is a simplified version.

## Constructing nodes

If the `pass` IR contains fields from both `to` and `from` which can make constructing these nodes quite verbose.
The `NodeConstructMacros` derive adds helper macros to construct these nodes without having to specify all fields.
Let's see how these macros are used in the `self_types` pass.
This adds a `self_type` field to type definitions, which will allow them to do things like check the `self_type.is_used_as_error` field.

```rust
// Use NodeConstructMacros to define the macros used below
#[derive(Node, NodeConstructMacros)]
struct Interface {
    ...
}

#[derive(Node, NodeConstructMacros)]
enum Type {
    ...
}

pub fn step(module: &mut Module) -> Result<()> {
    // This step inputs `Module` so that it can get the module name, then uses the `visit_mut`
    // method to further descend into the node tree and mutate the `Interface` nodes
    //
    // This is not really related to the macros, but it's a common pattern.
    let module_name = module.name.clone();
    module.visit_mut(|int: &mut Interface| {
        // Use `TypeNode!` to construct a `TypeNode` value.
        // Fields missing from the initializer, like `is_used_as_error`, will be set to empty
        // Later passes will fill them in.
        int.self_type = TypeNode! {
            // Similary, use `Type_Interface!` to construct a `Type::Interface` value.
            ty: Type_Interface! {
                module_name: module_name.clone(),
                name: int.name.clone(),
                imp: int.imp.clone(),
            },
        };
    });
    Ok(())
}
```

## AsRef

The `uniffi_pipeline` crate also defines an `AsRef` derive macro, which can be very useful for IRs.

Many nodes in the general IR contain a `TypeNode`.  Language IRs will typically add a
`TypeNode::ffi_converter` field which stores the name of the FFI converter class used to lower/lift
those nodes. If we can define `AsRef<TypeNode>` for each of those nodes, we can implement a filter
that works for all of them:

```rust
mod filters {
    fn lift_fn(node: AsRef<TypeNode>) -> Result<String> {
        Ok(format!("{}.lift", node.as_ref().ffi_converter()))
    }
}
```

The `AsRef` derive macro can be used to easily define these AsRef trait impls:

```rust
#[derive(Node, AsRef)]
pub struct Argument {
    pub name: String,
    // This `#[as_ref]` attribute generates an `AsRef<TypeNode>` impl that returns `&arg.ty`
    #[as_ref]
    pub ty: TypeNode,
}

// There's no blanket `impl AsRef<T> for <T>`, but you the AsRef macro can define one.
#[derive(Node, AsRef)]
#[as_ref]
pub struct TypeNode {
    pub ty: Type,
    pub ffi_converter: String,
}
```

## Peeking behind the curtains with the `pipeline` CLI

Use the `pipeline` subcommand from any UniFFI CLI inspect IR data at various stages of the pipeline.

* Build a UniFFI crate that you'd like to inspect, for example `cargo build -p uniffi-example-arithmetic`
* Run the [uniffi-bindgen CLI](../tutorial/foreign_language_bindings.md), with these arguments `pipeline --library path/to/library.so_a_or_dll [language]`
  * For example, in the UniFFI repo, `cargo run -p uniffi-bindgen-cli -- pipeline --library target/debug/libarithmetical.so python`

This will print out the IR at each point in the IR transformation process:
the initial, general, and Python IRs, each pass step and the conversions between into/from the pass IRs.
The first and last printout will contain the full IRs, in the middle will be a diff from the previous IR.

This is a lot of data.  Use CLI flags to reduce it to a reasonable amount:

* `-s`, `--step` only show a single step.
  * Use `-s last` to only show the last step in the process, which can be usefull when you're adding step functions and want to see their effects.
- `-t`, `--type` to only show one node type (e.g. `-t Record` or `-t Interface`).
- `-n`, `--name` to only show with nodes with a specific name

You can test this out yourself by running the following command to follow the `add` function as it moves throw the IR pipeline:

`cargo run -p uniffi-bindgen-cli -- pipeline --library target/debug/libarithmetical.so  python -t Function -n add` 
