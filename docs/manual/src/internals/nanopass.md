# The UniFFI nanopass system

Rendering foreign bindings can be modeled as a compiler pipeline.
We start with code as input, then generate code as output.
Even though we're generating foreign bindings code rather than machine code, many of the same techniques can apply.

UniFFI uses a [nanopass-style](https://nanopass.org/) pipeline to render the foreign bindings, which can be summarized as:

 - Start with an intermediate representation (IR) that reflects the exported Rust interface.
 - Transform the IR using many small steps.
 - Finish with an IR that's ready to be rendered using [rinja](https://rinja.readthedocs.io/) templates.
 - Render the templates to generate the foreign bindings.

## Example

Let's suppose we want to export this Rust code:

```rust
/// Repeat a string `n` times
#[uniffi::export]
pub fn repeat_str(text: String, n: u32) -> String {
    ...
}
```

The `uniffi::export` macro will generate a FFI function that calls `repeat_str`.
Bindings generators need to generate code that:
  * Lowers all arguments into FFI types
  * Calls the Rust FFI function
  * Lifts the return value back into a high-level type
  * (Much more is involved, but we'll gloss over that to keep the example simple).

The Python bindings code might look like this:

```
def repeat_string(text: str, n: int): str {
    result = _UNIFFI_LIBRARY.uniffi_ffi_func_repeat_str(
        _UniffiFfiConverterString.lower(text),
        _UniffiFfiConverterInt.lower(n),
    )
    return _UniffiFfiConverterString.lift(result)
}
```

* `_UNIFFI_LIBRARY` is a global object that represents the dynamic Rust library
* `_UNIFFI_LIBRARY.uniffi_ffi_func_repeat_str` is the generated FFI function for `repeat_str`
* `_UniffiFfiConverter*` are objects that can lower Python into FFI types and vice-versa.

## The Initial IR

The first form of the IR simply reflects the Rust interface without any additional data.
It's generated from the metadata, that the macro code stores in the dynamic library.

```rust
use uniffi_bindgen::nanopass::Node;

/// The Ir is just a list of metadata items at this point
///
/// All Ir nodes need to derive the `Node` trait.
#[derive(Node)]
pub struct Ir {
    metadata: Vec<Metadata>,
}

/// Enum covering all the possible metadata types
#[derive(Node)]
pub enum Metadata {
    /// Metadata item for functions
    Func(Function),
    /// Other metadata items that we won't use in this example.
    Record(RecordMetadata),
    Enum(EnumMetadata),
    Constructor(ConstructorMetadata),
    Method(MethodMetadata),
    /// ... more items below
}

#[derive(Node)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Argument>,
    pub return_type: Option<Type>,
}

#[derive(Node)]
pub struct Argument {
    pub name: String,
    pub ty: Type,
}

/// High-level type in the interface
#[derive(Node)]
pub enum Type {
    UInt32,
    String,
    // ... more variants below
```

### Node types

Types used in the IR must derive the `Node` trait using the derive macro.
The following types are allowed:
* Structs
* New-types (tuples with a single field)
* Enums where all variants are one of these:
    * Struct-style
    * New-type style
    * Unit types

## The Final IR

The final form of the IR is something that we can pass into a `rinja` template to easily render the above code.

```rust
#[derive(rinja::Template, Node)]
pub struct Ir {
    /// Functions have been moved out of the metadata list and into their own field.
    functions: Vec<Function>,
    //.. more fields below
}

#[derive(Node)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<Argument>,
    /// Objects now store `TypeNode` which has extra fields compared to `Type`.
    pub return_type: Option<TypeNode>,
    /// Name of the FFI function to call
    pub ffi_function: String,
}

#[derive(Node)]
pub struct Argument {
    pub name: String,
    pub ty: TypeNode,
}

#[derive(Node)]
pub struct TypeNode {
    /// The original type
    pub ty: Type,
    /// The Python type name
    pub name: String,
    /// Name of the FFI converter object for this type
    pub ffi_converter_name: String,
}
```

## Transforming the IR

To get from the initial IR to the final IR, we'll create a pipeline that consists of a series of small passes.
Each pass can transform the IR -- transforming its types.

```
use uniffi_bindgen::nanopass::{Ir, Pipeline};

fn render_python_bindings(initial_ir: Ir) -> Result<String> {
    // Create a pipeline by starting with `default` and adding passes.
    // Each pass consists of a name and a transformer function
    let pipeline = nanopass::Pipeline::default()
        .transform("move function metadata", move_function_metadata)
        .transform("add ffi functions", add_ffi_function)
        .transform("transform types to type nodes", type_to_type_node);

    // Send the IR through the pipeline and render the result
    let final_ir = pipeline.process(initial_ir)?;
    Ok(final_ir.render()?)
}
```

Transformer functions input an IR node and outputs an IR node.
The input and output nodes can be different, which will transform the types in the IR.

### Pass: transform types to type nodes

```rust
use anyhow::Result;
use uniffi_bindgen::nanopass::Node;

/// Input the `Type` from the initial IR and output a `TypeNode` from the final IR.
///
/// This will transform all `Type` instances in the IR to a new type.
/// This transforms the `Argument`, `Function` and `Ir` types as well, since these contain a
/// `Type` -- either directly or transitively.
fn type_to_type_node(ty: Type) -> Result<TypeNode> {
    let name = match ty {
        Type::UInt32 => "int".to_string()
        Type::String => "str".to_string()
        // .. more cases below.
    };
    let ffi_converter_name = match ty {
        Type::UInt32 => "_UniffiFfiConverterInt".to_string()
        Type::String => "_UniffiFfiConverterString".to_string()
        // .. more cases below.
    };
    Ok(TypeNode { ty, name, ffi_converter_name })
}
```
### Pass: add FFI functions

This pass transforms the `Function` type by adding the `ffi_function` field.
When writing these passes, it's usually best to only define the fields needed in the transform.

```rust
use anyhow::Result;
use uniffi_bindgen::nanopass::Node;

/// Input type for the transform
#[derive(Node)]
#[node(name=Function)]
struct Function1 {
    /// This is used to derive the FFI function name
    name: String,
    // No other fields are listed, which means they will be left untouched by the pass.
    // Not listing the fields can make transforms easier to read since readers can focus on the
    // action without being distracted by what's not changing.
    //
    // It also makes transforms easier to compose.
    // This transform would work even if a previous pass transformed the other fields.
}

/// Output type for the transform
///
/// Note that the input and output types both have the `#[node(name=Function)] attribute.
/// This is needed because we have two different Rust types that operate on the IR type `Function`.
#[derive(Node)]
#[node(name=Function)]
struct Function2 {
    name: String,
    ffi_function: String,
}

fn add_ffi_function(input: Function1) -> Result<Function2> {
    let ffi_function = format!("_UNIFFI_LIBRARY.uniffi_ffi_func_{}", input.name);
    Ok(Function2 { name, ffi_function })
}
```

### Pass: move function metadata

This pass moves function metadata to a new `Ir::Function` field and removes the `Metadata::Function` variant.
It's more complicated than the last ones, since multiple types are involved in the transformation.

```rust
use anyhow::Result;
use uniffi_bindgen::nanopass::{Node, Unknown};

// The IR will be getting a new `functions` field
#[derive(Node)]
#[node(name=Ir)]
pub struct Ir1 {
    metadata: Vec<Metadata1>,
}

#[derive(Node)]
#[node(name=Ir)]
pub struct Ir2 {
    metadata: Vec<Metadata2>,
    functions: Vec<Function>,
}

// Functions will be staying the same, in fact we don't care about the field data at all
//
// Since our transform inputs `Ir` rather than inputting `Function` directly, we need to either
// list all fields explicitly or use an `Unknown` field.  `Unknown` will store all data not
// explicitly listed. This allows the nanopass pipeline to track this data as the `Function` gets
// moved around.
//
// Since this only has a single field, we can use a new-type
#[derive(Node)]
pub struct Function(Unknown);

// Metadata will losing a variant
#[derive(Node)]
#[node(name=Metadata)]
pub enum Metadata1 {
    Func(Function),
    /// Unknown variant, this stores all other variants.
    Unknown(Unknown),
}


#[derive(Node)]
#[node(name=Metadata)]
pub enum Metadata2 {
    Unknown(Unknown),
}

/// Extract functions from `metadata` and move them into their own field
fn move_function_metadata(input: Ir1) -> Result<Ir2> {
    let mut functions = vec![];
    let metadata = input.metadata
        .into_iter()
        .filter_map(|meta| {
            match meta {
                Metadata1::Function(f) => {
                    functions.push(f);
                    None
                }
                Metadata1::Unknown(u) => Some(Metadata2::Unknown(u)),
            }
        })
        .collect();
    Ok(Ir2 { functions, metadata })
}
```

### When do you need to use Unknown?

* The struct is missing fields or an enum is missing variants.
* It's a child or descendant of the output type
* Note: this includes recursive types, which are descendants of themselves.

### node_pairs!

The `node_pairs!` macro can be used as a shortcut to create a set of input and output node types.
The `Ir1`/`Ir2` and `Metadata1`/`Metadata2` types could have been created with:

```rust
use uniffi_bindgen::nanopass::node_pairs;

node_pairs! {
    pub struct Ir {
        metadata: Vec<Metadata>,
        +functions: Vec<Function>,
    }

    pub enum Metadata {
        -Func(Function),
        Unknown(Unknown),
    }
}
```

In addition to reducing boilerplate, `node_pairs!` can often clarify the intent of a transform.

### Other pipeline pass types

* Use `pipeline::mutate` for passes that mutates the data, but don't change any types.
  A common use case for this is changing the naming style of items, for example camelCasing arguments.
* Use `pipeline::pass` for general-purpose passes.
  These input the `Ir` and can run any of it's methods, including `Ir::transform`.
  Common use cases are passes that runs two transforms or ones that call `Ir::iter_nodes` to collect some data, then calls `Ir::transform` to run a transform that uses that data.

## Conclusion

That finishes our our trip through the nanopass system.
Of course, this was a toy example and real bindings generators create more more complex pipelines.
However, the nanopass system is good at breaking down complex problems into smaller ones.
If you want to learn more, take a look at the Python pipeline code to see how these concepts work in the real world.
