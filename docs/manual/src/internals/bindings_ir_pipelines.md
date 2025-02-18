# Bindings IR Passes

Bindings IR passes are used to transform different [intermediate representations of the generated bindings](./bindings_ir.md).

* `uniffi_pipeline` defines the foundational code
* `uniffi_bindgen` defines the general pipeline -- this converts `uniffi_meta` metadata to the general IR
* Each language defines their own pipeline, which extends the general pipeline to output their language-specific IR.

## Nodes

Types inside an are are called "nodes" and they must derive a `Node` trait impl.
The top-level node is always named `Root`.
When this doc talks about converting between two IRs, this means converting between the root nodes of those IRs.

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

## Modules structure

IRs usually have a module dedicated to them with the following structure:
* `mod.rs` -- Top-level module.  Also, defines the pipeline to convert to this IR.
* `nodes.rs` -- Node definitions
* *other submodules* -- Pipeline pass funcions.  These are conventionally named `pass()`.

## Defining IRs

* Start with an existing IR, let's call it `from_ir`
* Define a new module for your IR, let's call it `into_ir`
* Copy the `from_ir/nodes.rs` to `into_ir/nodes.rs`
* Add new fields to the structs in `into_ir::nodes`
  * For enums, consider using `#[node(wraps)]` to wrap them with a struct. The wrapper struct can have extra fields added.
* Define pipeline passes to populate the new fields, using the existing fields.  For example, `into_ir/mod.rs` might define a pipeline like this:

```rust
pub fn pipeline(): Pipeline<initial::Root, Root> {
    // Start with `from_ir's` pipeline, this converts `initial::root` to `from_ir::Root`.
    from_ir::pipeline()
        // Convert to `into_ir::Root`.  Each added field will be initialized to an empty value.
        .convert_ir_pass::<Root>()
        // Add passes to populate the new fields and/or mutate the IR in general
        .pass(foo::pass)
        .pass(bar::pass)
        .pass(baz::pass)
}
```

* Define the pipeline passes.
  These input a node type in the IR and mutate all instances of that node.
  `visit()` and `visit_mut()` are extremely helpful here.
  For example, the general IR pass that adds FFI types to types looks like this:

```rust
// Visiting all `Module` nodes
pub fn pass(module: &mut Module) -> Result<()> {
    // Save a copy of the module name, then visit all `TypeNode` nodes.
    // Note: TypeNode wraps the `Type` enum from the initial IR
    let module_name = module.name.clone();
    module.visit_mut(|node: &mut TypeNode| {
        // Derive the FfiType from the type and module name
        node.ffi_type = generate_ffi_type(&node.ty, &module_name);
    });
    Ok(())
}
```

See `uniffi_bindgen::pipeline::general` for examples.

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

pub fn pass(module: &mut Module) -> Result<()> {
    // This pass inputs `Module` so that it can get the module name, then uses the `visit_mut`
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

This will print out:

* The initial IR
* The diff after each pass
* The final IR

This is a lot of data.  Use CLI flags to reduce it to a reasonable amount:

* `-p`, `--pass` only show a single pass.
  * Use `-p final` to only show the final pass in the process, which can be usefull when you're adding pass functions and want to see their effects.
- `-t`, `--type` to only show one node type (e.g. `-t Record` or `-t Interface`).
- `-n`, `--name` to only show with nodes with a specific name

You can test this out yourself by running the following command to follow the `add` function as it moves throw the IR pipeline:

`cargo run -p uniffi-bindgen-cli -- pipeline --library target/debug/libarithmetical.so  python -t Function -n add` 
