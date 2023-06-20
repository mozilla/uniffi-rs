# Allowing more types to be used as errors.

* Status: proposed
* Deciders: Hopefully consensus on [#1613](https://github.com/mozilla/uniffi-rs/pull/1613)

## Current UniFFI error handling

UniFFI functions currently must report error returns via an `Enum` - ie, the `E` in `Result<T, E>` must be an Enum.

This ADR proposes moving towards a model where more types can be an `E`, and specfically, proposes allowing any uniffi "interface".

Assuming this isn't controversial, it will also make decisions about how such errors should be represented in the foreign bindings we maintain.

### The current situation
When an `Enum` is used as an error, on the foreign side it's not an `Enum` at all - it's a distinct type -
it derives from the local `Exception` class and isn't usable as the actual `Enum` behind it.

### The options for interfaces.
Consider the UDL
```idl
interface ErrorDetails { string message(); };
[Throws=ErrorDetails] void do_it();
``````
and Rust
```rust
struct ErrorDetails {}
impl ErrorDetails {
    fn message(&self) -> String { todo!() }
}
fn do_it() -> Result<(), ErrorDetails> { Err(ErrorDetails {}) }
```

How should this be captured on the foreign side?

### A) Create an "error only" type

This means an interface used as an error behaves a lot like an enum used as en error. It means:
* There's no "non error" version of the interface. (just like there's no non error version of an enum declared as an error)
* A struct simply appearing as an error changes how the foreign side sees that struct significantly.
* The type can't be used exactly as it could in non-error contexts.

ie, it's a bit of "either or, but not both". But it makes for more natural foreign code:

```
try:
    do_it()
except ErrorDetailsException as e:
    # There's no `ErrorDetails` type in this world.
    # ie, `e' is an  `ErrorDetailsException`
    # `ErrorDetailsException` derives from `Exception`
    e.message()
```

### B) Create an original type and an error type

We create 2 types - the `interface` itself unchanged from how the struct would be generated now,
and another exception "wrapper" for the type.
It means you *catch* a slightly different name from the interface, but the non-error version of the interface is what you deal with.

```
try:
    do_it()
except ErrorDetailsException as e:
    # There are 2 types here:
    # `e` is an `ErrorDetailsException`
    # `e.inner` (or any other name) is an `ErrorDetails` - does not derive from `Exception`
    # We can make it look natural via delegation.
    e.message()
    # But also use the original interface
    HandleErrorReportingProblem(e.inner)
```

To clarify, you could imagine generated Python code looking something like:
```
class ErrorDetails:
    ...
class ErrorDetailsException(Exception):
    ...
```
### C) Create and original type with an error type nested within the original

Very similar to the above - there are still 2 unique types created, but only one type is a top-level
type - the error type is created inside the exception.

The code catching exceptions might look something like:
```
try:
    do_it()
except ErrorDetails.Exception as e:
    # same block above as for B
```

ie, there's no top-level `ErrorDetailsException` type, it's accessed via `ErrorDetails.Exception`

To clarify, you could imagine generated Python code looking something like:
```
class ErrorDetails:
    # snip regular interface.
    class Exception(Exception):
        # snip error specific code
```

## Decision Outcome

Chosen option: "A) Create an "error only" type" - it's more natural to foreign authors, and
the use-cases and pro/con list can't identify a compelling benefit to having multiple objects.

## Pros and Cons of the Options

### 1: Do nothing - don't support interfaces as errors.

Con: Seems a bit negative :)

### 2: Implement Option A above

- Pro: no magic, there's exactly 1 type for the type in the UDL/macro
- Con: That type gets "tainted" by being used as an error. Using the interface in other contexts is less natural.
- Con: It might be tricky to use as an external type - all external consumers must use as an error, or none of them must.

### 3: Implement Option B above:

- Pro: a more "pure" system - the interface can be used naturally in contexts other than being an error.
- Pro: It mirrors Rust features in that it "adds" a capability rather then transforms it; it's more natural from a Rust viewpoint.
- Con: It does not mirror the standard behavior of any of our bindings; it's less natural from a foreign viewpoint.
- Con: The wrapping might get in the way, and it's more complexity for a possibly non-existing use-case.

### 4: Implement Option C above:

- Pro and Cons roughly the same for Option B plus:
- Pro: less global types
- Pro: the relationship between the interface and error is cleaner
- Con: Might feel less natural to foreign authors.

### 5: Something else?

* Even considering future error types (eg, `dictionary` - ie, a struct with only data),
  the use-cases can't identify a benefit to having multiple objects.

## Links <!-- optional -->

* [PR for this ADR](https://github.com/mozilla/uniffi-rs/pull/1613)
* [PR for an implementation with support for Python](https://github.com/mozilla/uniffi-rs/pull/1662)
