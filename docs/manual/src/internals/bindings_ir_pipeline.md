# Bindings IR Pipeline

**Note:** the Bindings IR is currently an experiment.
It's checked in so that we can use it for the gecko-js external binding generator.
Our current recommendation for other external bindings authors is to avoid using it for now since we haven't fully committed to this new system and expect it to change.

The Bindings IR pipeline is used to transform different [intermediate representations of the generated bindings](./bindings_ir.md).

* The foundational code lives in the `uniffi_pipeline` crate.
  This defines things like the `Node` trait.
* The macro code lives in the `uniffi_internal_macros` crate.
  This defines things like the `Node` derive macro.
* `uniffi_bindgen` defines the general pipeline.
  This converts `uniffi_meta` metadata to the initial IR then converts that to the general IR.
* Finally, each language defines their own pipeline, which extends the general pipeline and outputs a language-specific IR.

## Nodes

Types inside an IR are called "nodes" and they must derive a `Node` trait impl.
The top-level node is always named `Root`.
When this document talks about converting between two IRs, this means converting between the root nodes of those IRs.

`Node` trait provides functionality to:

* Traverse the node tree.  `Node::visit()` and `Node::visit_mut()` allows you to visit all
  descendants with a given type.
* Convert between any two nodes using `Node::try_from_node`.  This conversion is described in the
  next section.

## Node conversions

`Node::try_from_node` attempts to convert a node from one IR to the corresponding node from another IR.
For example, `from_ir::Node` will be converted to `into_ir::Node` using the following rules:

* Any fields in `into_ir::Node`, but not `from_ir::Node` are added using `Default::default`, which the `Node` derive macro also derives.
* Any fields in `from_ir::Node`, but not `into_ir::Node` are ignored
* Any fields in both are recursively converted using `Node::try_from_node`.
* `try_from_node` is automatically derived using the `Node` derive macro.
  The conversion can be customized using macro attributes.

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
    /// Any other fields will be initialized to their default values.
    ///
    /// This is mostly used as a way to add fields to enum types.
    /// Wrap the enum with a struct, then add fields to that struct.
    /// For example, `TypeNode` wraps the `Type` enum from `uniffi_meta`.
    #[node(wraps)]
    wrapped: WrappedNode,
}
```

### Field Conversion order

Fields are converted in declaration order.
This matters when a field is listed twice with one of them using `#[node(from(<name_of_the_other_field>)]`.
In this case the first field declared will be initialized with the data from the previous pass and the second field will be initialized to the default value.

You can take advantage of this to convert an optional value into a non-optional value.
For example:

```rust
#[derive(Node)]
pub struct SourceNode {
    name: Option<String>
}

#[derive(Node)]
pub struct DestNode {
    /// This field will be set to `SourceNode::name`
    #[node(from(name))]
    name_from_previous_pass: Option<String>
    /// This field will be a non-optional version of `SourceNode::name`.
    /// It will be initialized to an empty string, then a pipeline pass will populate it using
    /// `name_from_previous_pass` combined with logic to handle the `None` case.
    name: String,
}
```

## Module structure

Each IRs will typically have a module dedicated to them with the following structure:

* `mod.rs` -- Top-level module.  This is where the pipeline for the IR is defined.
* `nodes.rs` -- Node definitions.
* *other submodules* -- Define pipeline pass functions.  These are named `pass()` by convention.

## Defining IRs

* Start with an existing IR, let's assume it lives in the `from_ir` module.
* Define a new module for your IR, let's call it `into_ir`
* Copy the `from_ir/nodes.rs` to `into_ir/nodes.rs`
* Add new fields to the structs in `into_ir::nodes`
* Define pipeline passes to populate the new fields, using the existing fields.  For example, `into_ir/mod.rs` might define a pipeline like this:

```rust
// The output type is a pipeline that converts from `initial::Root` to the `Root` node from this IR.
pub fn pipeline() -> Pipeline<initial::Root, Root> {
    // Start with `from_ir's` pipeline.  This converts `initial::root` to `from_ir::Root`.
    from_ir::pipeline()
        // Convert to `into_ir::Root`.
        // This will use the logic from the Node conversions section above.
        .convert_ir_pass::<Root>()
        // Add passes to populate the new fields, mutate existing fields, etc.
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
    // Visit all `TypeNode` instances that are descendants of `Module`.
    module.visit_mut(|node: &mut TypeNode| {
        // Derive the FfiType from the type and module name
        node.ffi_type = generate_ffi_type(&node.ty, &module_name);
    });
    Ok(())
}
```

See `uniffi_bindgen::pipeline::general` for examples.

## Constructing nodes with partial field data

Passes that construct new nodes often only want to specify partial field data and let a later pass populate the rest of the fields.
Use the `Default::default()` to handle this case, which the `Node` derive also implements.
Here's how this works in the `callables` pass:

```rust
pub fn pass(root: &mut Root) -> Result<()> {
    root.visit_mut(|func: &mut Function| {
        func.callable = Callable {
            // Most of the fields are simply copied from `Function`
            name: func.name.clone(),
            is_async: func.is_async,
            kind: CallableKind::Function,
            arguments: func.inputs.clone(),
            return_type: ReturnType! {
                ty: func.return_type.clone().map(|ty| TypeNode! { ty }),
            },
            throws_type: ThrowsType! {
                ty: func.throws.clone().map(|ty| TypeNode! { ty }),
            },
            checksum: func.checksum,
            // However, `async_data` and `ffi_func` are derived in a later pass.
            // Use `default()` to create placeholder values for now
            ..Callable::default()
        }
    });
    // ... repeat for methods and constructors
    Ok(())
}
```

## Peeking behind the curtains with the `pipeline` CLI

Use the `pipeline` subcommand from any UniFFI CLI to inspect IR data at various stages of the pipeline.

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

Alternatively, if you want to see the full IR for each pass, you can use `--no-diff` to print it out.
Piping to a pager like `less` is highly recommended in this case.

You can test this out yourself by running the following command to follow the `add` function as it moves through the IR pipeline:

`cargo run -p uniffi-bindgen-cli -- pipeline --library target/debug/libarithmetical.so python -t Function -n add | less`
