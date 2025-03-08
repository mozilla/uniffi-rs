# Bindings Ir Passes

Bindings IR passes are used to transform different [intermediate representations of the generated bindings](./bindings_ir.md).

## Nodes

Types inside an are are called "nodes" and they must derive a `Node` trait impl.
The top-level node that contains all others is always named `Root`.

The IR can be thought of as a directed graph, and the `Node` trait provides functionality to traverse that graph.
For example, `Node::visit()` and `Node::visit_mut` allows you to visit all descendants with a given type.

## The IRs for a pass

Each pass involves 3 different IRs:

- The `from` IR contains the nodes we're transforming from
- The `to` IR contains the nodes we're transforming into
- The `pass` IR contains the nodes we use use to implement the pass.
  These nodes are essentially the union of the `from` and `to` IRs nodes.

Each IR lives in a separate module, with `pass` module being a child of the `to` module.

For example, one thing the initial IR -> general IR pass does is convert/normalize module paths into module names.
This means, we'll have 3 different `Type` enums:

```rust
mod initial {
    // Types in the initial IR have a `module_path` field
    //
    // There's more variants and fields in the actual definition, but we'll
    // just focus on module_path/module_name
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
    // Types in the general IR have a `module_name` field
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
        // Types in the general::pass IR have both `module_path` and `module_name`
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
    }
}
```

## Pass steps

A pass is divided into a series of step functions, where each step is responsible for a specific part of the pass.
Step function input `&mut Node` for any node type in the IR and transform all nodes of that type.
Steps usually set values for previously empty fields.

For example, the step to handle the module_path -> module name transformation could look like this:

```rust
pub fn step(root: &mut pass::Type) -> Result<()> {
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
```

This is why the `pass` IR contains fields from both the `from` and `to` IRs:
the pass can access both the old fields and new fields while transforming the data.

Note: Module name normalization is actually done as part of a larger step that moves modules into a tree.
This example is a simplified version.

## Constructing nodes

If the `pass` IR contains fields from both `to` and `from` then do you need to specify each one of them when constructing a node?
This would be pretty annoying, so `ir_pass!` sets up some macros for you to use that will fill in any missing fields with empty values.
Let's see how these macros are used in the `self_types` pass.
This adds a `self_type` field to type definitions, which will allow them to do things like check the `self_type.is_used_as_error` field.

```rust
pub fn step(module: &mut Module) -> Result<()> {
    // This step inputs `Module` so that it can get the module name, then uses the `visit_mut`
    // method to further descend into the node tree and mutate the `Interface` nodes
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

## Converting between IRs

Here's full process for an IR pass:
* Convert from the `to` IR to the `pass` IR
* Execute all pass steps
* Convert from the `pass` IR to the `to` IR


The `ir`, `ir_pass!` and `Node` macros work together to generate an automatic conversion for the first and last bullet using the following logic:
* Each node is converted to the node with the same name in the next IR (`general::Record` is converted to `initial::Record`).
* If a field is in both IRs, then it's converted using the `FromNode` trait.
  This is what the macros automatically derive, but you can also implement `FromNode` manually -- see below.
* If a field is added, we use the result of `Node::empty()` to initialize it.
  * Primitive types (`String`, `u8`, `bool`, etc) will be initialized to `Default::default()`.
  * Structs/enum variants will have each field initialized to the `Node::empty()` value.
  * Enums will be initialized to their first variant.
* If a variant is removed, then the conversion code will generate a runtime error.
  To avoid this, the pass steps must make sure to transform all instances of the removed variant to one that's not removed.

There are a couple advanced techniques that you can use to customize this conversion:
* It's possible to automatically map values of one type to another by manually implementing the `FromNode` trait.
  * For example, the general IR maps all `Type` values to be `TypeNode` values.
    See `uniffi_bindgen/src/ir/general/pass/mod.rs` for how this works.
* You can annotate a field with `#[pass_only]` to make it only present in the pass IR.
  This allows you to store temporary data in that field during one pass step, use it in a future step, then remove the field when converting to the `to` IR.
  * For example, see the `Callable::is_async` field, which is copied from the `is_async` field of `Function`/`Method`/`Construct`.
    However, a later pass adds the `async_data` field which makes `is_async` redundant.
    By using `#[pass_only]`, we can access that data in the step functions, but not have it present in the general IR.

## The macros

The `ir` module provides several macros to help implement IRs and IR passes:

- The `Node` derive macro is used to implement the `Node` trait.
- The `ir!` macro wraps the type definitions for the `from` and `to` IRs, which allows the `ir_pass!` macro to auto-generating the `pass` IR.

See the `ir::initial`, `ir::general` and `ir::general::pass` modules for how these are used.

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
