# Bindings IR Pipeline

**Note:** the Bindings IR is currently an experiment.
It's checked in so that we can use it for the gecko-js external binding generator.
Our current recommendation for other external bindings authors is to avoid using it for now since we haven't fully committed to this new system and expect it to change.

The Bindings IR pipeline is used to transform different [intermediate representations of the generated bindings](./bindings_ir.md).

* The foundational code lives in the `uniffi_pipeline` crate.
  This defines things like the `Node` and `MapNode` traits.
* The macro code lives in the `uniffi_internal_macros` crate.
  This defines derive macros for `Node` and `MapNode`.
* `uniffi_bindgen` defines the general pipeline.
  This converts `uniffi_meta` metadata to the initial IR then converts that to the general IR.
* Finally, each language defines their own pipeline, which extends the general pipeline and outputs a language-specific IR.

## Nodes

Types inside an IR are called "nodes" and they must derive a `Node` trait impl.
The top-level node is always named `Root`.
When this document talks about converting between two IRs, this means converting between the root nodes of those IRs.

`Node` trait provides functionality for walking the IR tree.

* `Node::visit()` and `Node::try_visit()` allows you to visit all descendants with a given type.
* `Node::has_descendant()` tests if a predicate is true for any descendant.

These are useful when you want to populate a node field using it's descendants.  For example,
building up the FFI definitions by visiting all functions/methods inside a namespace.

Node methods input a closure that inputs a node type, for example `root_node.visit(|func: Function| ...)`.
`visit()` will walk the node tree and call that closure with all matching nodes.  The methods panic
if there are no descendant types with the input node type.  This checks the types involved, not the
values.  If there is a `Vec<Function>` field then `visit()` will not panic, even if the vec is empty.

## MapNode

The `MapNode` trait is used to convert from one node to another and is defined like this:

```rust
pub trait MapNode<Output, Context> {
    fn map_node(self, context: &Context) -> Result<Output>;
}
```

* `Self` is a node type from the previous IR
* `Output` is a node type from current IR
* `Context` is a generic context type which can be used to pass data to descendant nodes.

Here's an example of how a `MapNode` implementation might look:

```rust
impl MapNode<Namespace, Context> for prev_ir::Namespace {
    fn map_node(self, context: &Context) -> Result<Namespace> {
        // Create a new context for our descendants
        let mut child_context = context.clone();
        let context = &mut context;
        context.set_namespace_name(&self.name);
        // Map the previous `Namespace` node into the current version
        Ok(Namespace {
            // New fields are usually populated by calling a function
            ffi_definitions: generate_ffi_definitions(&self, context)?,
            // Existing fields are usually converted using recursive `map_node` calls.
            name: self.name,
            functions: self.functions.map_node(context)?,
            type_definitions: self.type_definitions.map_node(context)?,
            // ...
        });
    }
}
```

`MapNode` can be derived automatically.  For the previous impl that would look like:

```rust
#[derive(Node, MapNode)]
// Use `#[map_node(from([type_name]))]` to declare the type we're mapping from
#[map_node(from(prev_ir::Namespace))]
// Use `#[map_node(update_context([expr]))]` update the context for descendant `map_node()` calls.
#[map_node(update_context(context.set_namespace_name(&self.name)))]
pub struct Namespace {
    // Use `#[map_node(expr)]` to manually define the expression to populate a field.
    #[map_node(generate_ffi_definitions(&self, context)?)]
    ffi_definitions: Vec<FfiDefinition>,
    // If no `#[map_node]` attribute is present, then fields will be mapped using recursive
    // `map_node()` calls.
    name: String,
    functions: Vec<Function>,
    type_definitions: Vec<TypeDefinition>,
    // Note: The derived impl will convert fields in the order they're defined in.  This means we
    // need to put `ffi_definitions` first, so that we can take a reference to `self` before `self`
    // is deconstructed for the `map_node` calls.
}
```

The `MapNode` derive macro supports a few other attributes:

```rust
#[derive(Node, MapNode)]
#[map_node(from(prev_ir::TypeNode))]
// When the type itself has the `#[map_node([path-to-function])]` attribute, then that function will
// be used for the entire mapping logic. In this case `MapNode::map_node` will forward the call
// `types::map_type_node`.
//
// Use this as an escape hatch for mappings that can't be auto-generated.
#[map_node(types::map_type_node)]
pub struct TypeNode {
    //...
}

#[derive(Node, MapNode)]
#[map_node(from(uniffi_meta::Type))]
pub enum Type {
    // Use `#[map_node(from)]` on variants/fields when they've been renamed from the previous IR
    #[map_node(from(Object))]
    Interface {
        #[map_node(from(prev_name_field))]
        name: String,
        // ...
    },
    // Use `#[map_node(added)]` for variants that have been added in this IR.  The generated code
    // will not try to map these variants since they didn't exist in the previous IR
    #[map_node(added)]
    External {
        // ...
    },
}
```

A common pattern is wanting mapping an enum to a struct so that we can add fields that apply to all
variants.  This can be achieved like this:

```rust
#[derive(Node, MapNode)]
#[map_node(from(uniffi_meta::Type))]
pub struct TypeNode {
    #[map_node(ffi_types::ffi_type(&self, context)?)]
    ffi_type: FfiType,
    #[map_node(self)]
    ty: Type,
}
```

Finally, if a type hasn't changed at all between IRs, then it can be re-used using the
`use_prev_node!` macro.  Note this only works if none of the fields have been changed either.

```rust
// Radix is a very simple Enum and doesn't change between IRs
use_prev_node!(uniffi_meta::Radix);
// Sometimes we want to map unchanged types using a function.  For example, applying rename
// logic to certain `Type` variants.  This can be achieved by specifying the map function as the
// second argument.
use_prev_node!(uniffi_meta::Type, types::map_type);
```
## Context types

The `Context` type provides a way for nodes to pass data from one node down to the `map_node()`
methods of descendant nodes. Here are some examples of how it's used:

* Passing the crate name down so it can be used to derive FFI function names.
* Passing the namespace name down so it can be used to determine which types are external.
* Passing the current type down so it can be used to populate `CallableKind::Method.self_type`.

`Context` can also be used to store data from outside the pass.  For example, to implement the
renaming logic in the general pass, it needs to know the key in the TOML file that contains the
rename map (`python`, `kotlin`, `swift` etc).  To handle this, the `general::Context` struct is
constructed with that key stored inside it.

Finally, `Context` types also act as marker types for the `MapNode` trait.  For example, the above
code wants to call `types::map_type` when mapping `Type` for one pass, but it shouldn't be called
for the next pass. This works because the `MapNode` impl is specific to the `Context` type for the
pass.

## Module structure

Each IR will typically have a module dedicated to them with the following structure:

* `mod.rs` -- Top-level module.
* `nodes.rs` -- Node definitions with `MapNode` derives.
* `context.rs` -- Defines the `Context` struct
* *other submodules* -- Define functions to implement the IR pass

## Assembling a pipeline

Pipelines are assembled by adding a series of passes that map one root node to another.

For example:

```rust
// Pipeline defined `uniffi_bindgen::pipeline::general`
//
// This is the shared start for all bindings pipelines.
// It maps `uniffi_bindgen::pipeline::initial::Root` to
// `uniffi_bindgen::pipeline::general::Root`
//
// Note that the context is constructed with `bindings_toml_key`.  This is how you can pass data
// from outside the pass into the `map_node` methods.
pub fn pipeline(bindings_toml_key: &str) -> Pipeline<initial::Root, Root> {
    new_pipeline()
        .pass::<Root, Context>(Context::new(bindings_toml_key))
}
```

```rust
// Pipeline defined `uniffi_bindgen::python::pipeline`
//
// This extends the general pipeline to map to
// `uniffi_bindgen::python::pipeline::Root`
pub fn pipeline() -> Pipeline<initial::Root, Root> {
    general::pipeline()
        .pass::<Root, Context>(Context::default())
}
```

## Starting a IR for binding generation

The first step is creating a skeleton for your IR:

* Define `pipeline` module in your crate.
* Define a `Context` type in `pipeline/context.rs`.  To start with, this can be an empty struct that
  derives `Default`.
* Define the IR nodes in `pipeline/node.rs`.  To start, this can just be `use_prev_node!` macros for
  each node from the general IR.
* Setup imports (optional).
   * `pipeline/mod.rs` file will normally import `nodes::*`, `anyhow::{anyhow, bail, Result}` and
     other items that are commonly used in the IR.
   * The other mods will normally import `super::*`.
   * This step is definitely not necessary, it's just how pipeline modules are typically setup.

From there you can evolve the code to match your needs.  For example:

* **Adding a new field**
    1. Copy and paste the type definition from the general IR into your `nodes.rs` file.
    2. Remove any `#[map_node]` attributes.
    3. Add `#[map_node(from(general::[NodeType]))` to the type itself.
    4. Add the new field with a `#[map_node([expression_to_generate_field])` attribute.
    5. Find any types that reference the changed type and repeat steps 1-3.
* **Mapping an enum to a struct**
    1. Define a new struct type.
       If your enum is named `Foo`, this will typically be named `FooNode`.
    2. Add a field that stores the original enum.  Wrap this with a `#[map_node(self)]` attribute.
    3. Add a new field, with a `#[map_node(expr)]` attribute
    5. Find any types that reference the changed type and repeat steps 1-3 from adding a new field.

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
