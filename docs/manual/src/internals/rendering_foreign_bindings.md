# Rendering Foreign Bindings

This document details the general system that UniFFI uses to render the foreign bindings code.

## The Askama template engine

Our foreign bindings generation is based off the [Askama](https://djc.github.io/askama/) template rendering engine.
Askama uses a compile-time macro system that allows the template code to use Rust types directly, calling their methods
passing them to normal Rust functions.

## Type matching

One of the main sources of complexity when generating the bindings is handling types.  UniFFI supports a large number of
types, each of which corresponds to a variant of the [`Type enum`](./api/uniffi_bindgen/interface/types/enum.Type.html).
At one point there was a fairly large number of "mega-match" functions, each one matching against all `Type` variants.
This made the code difficult to understand, because the functionality for one kind of type was split up.

Our current system for handling this is to have exactly 2 matches against `Type`:
  - One match lives in the template code.  We map each `Type` variant to a template file that renders definitions and
    helper code, including:
     - Class definitions for records, enums, and objects.
     - Base classes and helper classes, for example
       [`ObjectRuntime.kt`](https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/bindings/kotlin/templates/ObjectRuntime.kt)
       contains shared functionality for all the `Type::Object` types.
     - The FFIConverter class definition.  This handles [lifting and lowering
       types across the FFI](./lifting_and_lowering.md) for the type.
     - Initialization functions
     - Importing dependencies
     - See
       [`Types.kt`](https://github.com/mozilla/uniffi-rs/blob/main/uniffi_bindgen/src/bindings/kotlin/templates/Types.kt)
       for an example.
  - The other match lives in the Rust code.  We map each `Type` variant to a implementation of the `CodeType` trait that
    renders identifiers and names related to the type, including:
    - The name of the type in the foreign language
    - The name of the `FFIConverter` class
    - The name of the initialization function
    - See
      [`KotlinCodeOracle::create_code_type()`](https://github.com/mozilla/uniffi-rs/blob/470740289258e1f06171a976d8e15978f028e391/uniffi_bindgen/src/bindings/kotlin/gen_kotlin/mod.rs#L198-L230)
      for an example.

Why is the code organized like this?  For a few reasons:
  - **Defining Askama templates in Rust required a lot of boilerplate.**  When the Rust code was responsible for
    rendering the class definitions, helper classes, etc., it needed to define a lot of `Askama` template structs which
    lead to a lot of extra lines of code (see PR [#1189](https://github.com/mozilla/uniffi-rs/pull/1189))
  - **It's easier to access global state from the template code.**  Since the Rust code only handles names and
    identifiers, it only needs access to the `Type` instance itself, not the
    [`ComponentInterface`](./api/uniffi_bindgen/interface/struct.ComponentInterface.html) or the
    [`Config`](./api/uniffi_bindgen/struct.Config.html).  This simplifies the Rust side of things (see PR [#1191](https://github.com/mozilla/uniffi-rs/pull/1191)).
    Accessing the `ComponentInterface` and `Config` from the template code is easy, we simply define these as fields on
    the top-level template Struct then they are accessible from all child templates.
  - **Putting logic in the template code makes it easier to implement [external types](../udl/ext_types_external.md).**  For
    example, at one point the logic to lift/lower a type lived in the Rust code as a function that generated the
    expression in the foreign language.  However, it was not clear at all how to make this work for external types,
    it would probably require parsing multiple UDL files and managing multiple ComponentInterfaces.  Putting the logic
    to lift/lower the type in the `FFIConverter` class simplifies this, because we can import the external
    `FFIConverter` class and use that. We only need to know the name of the `FFIConverter` class which is a simpler
    task.

## Askama extensions/hacks

A couple parts of this system require us to "extend" the functionality of Askama (i.e. adding hacks to workaround its
limitations).

### Adding imports

We want our type template files to specify what needs to be imported, but we don't want it to render the import
statements directly. The imports should be rendered at the top of the file and de-duped in case multiple types require
the same import. We handle this by:

  - Defining a separate Askama template struct that loops over all types and renders the definition/helper code for them.
  - That struct also stores a `BTreeSet` that contains the needed import statements and has an `add_import()` method that
    the template code calls.  Using a `BTreeSet` ensures the imports stay de-duped and sorted.
  - Rendering this template as a separate pass.  The rendered string and the list of imports get passed to the main
    template which arranges for them to be placed in the correct location.

### Including templates once

We want our type template files to render runtime code, but only once.  For example, we only want to render
`ObjectRuntime.kt` once, even if there are multiple Object types defined in the UDL file.  To handle this the type
template defines an `include_once_check()` method, which tests if we've included a file before.  The template code then
uses that to guard the Askama `{% include %}` statement.  See [`Object.kt` for an
example](https://github.com/mozilla/uniffi-rs/blob/470740289258e1f06171a976d8e15978f028e391/uniffi_bindgen/src/bindings/kotlin/templates/ObjectTemplate.kt#L2)
