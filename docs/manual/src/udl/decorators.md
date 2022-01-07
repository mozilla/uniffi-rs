# Decorator objects

We'll be considering the UDL for an object called `DemoRustObject`:

```webidl
interface DemoRustObject {
    string do_expensive_thing();

    Sequence<i32> do_expensive_thing2();

    [Throws=RustyError]
    bool do_crashy_thing();

    [Throws=RustyError]
    u8 do_crashy_thing2();
}
```

`DemoRustObject` is a `struct` written in Rust, and uniffi generates a class of that name in the
foreign languages which forward method calls to the Rust implementation.

## Problem

Just by the names of these methods, we can see that the application might want to wrap the
`DemoRustObject` methods with some app-specific code.

* it may want to run `do_expensive_thing()` and `do_crashy_thing2()` in an `async` function that
  executes off the main thread.
* it may want to catch and report errors from `do_crashy_thing()` and `do_crashy_thing2()` and not
  propagate them upwards.

One way to handle this is writing a wrapper class for `DemoRustObject`:

```kotlin

import myapp.dispatchers.backgroundScope
import myapp.logging.errorReporter

fun <T> withErrorReporter(rustCall: () -> T) -> T? =
    try {
        return rustCall()
    } catch (e: Exception) {
        errorReporter(e)
        return null
    }
}

class WrappedDemoRustObject(val demoRustObject: DemoRustObject) {
    suspend fun doExpensiveThing(): String = withContext(backgroundScope) {
        demoRustObject.doExpensiveThing()
    }

    suspend fun doExpensiveThing2(): List<i32> = withContext(backgroundScope) {
        demoRustObject.doExpensiveThing2()
    }

    fun doCrashyThing(): bool? = withErrorReporter {
        demoRustObject.doCrashyThing()
    }

    fun doCrashyThing2(): u8? = withErrorReporter {
        demoRustObject.doConsequentialThing()
    }
}
```

However, this leads to a proliferation of hand-written boilerplate code:
  * Each time we add a method to `DemoRustObject`, we need to add a wrapper method to
    `WrappedDemoRustObject`.
  * Whenever we construct a `DemoRustObject` we need to wrap it with `WrappedDemoRustObject`.

## Decorators to the rescue

> Decorators are function wrappers, which get their name from [Python's decorator methods][py-decorators].
>
> Swift and Kotlin don't provide functionality to capture arbitrary `*args` and call a function with those same `*args`, so
> at this time, decorator objects aren't as powerful as Python decorators. Nevertheless, they can be still quite useful.

[py-decorators]: https://www.python.org/dev/peps/pep-0318/#on-the-name-decorator

### UDL

In the UDL we specify a decorator for each method with the `[Decorator={name}]` annotation.

```webidl
namespace demo {}

interface DemoRustObject {
    [Decorator=withBackgroundScope]
    string do_expensive_thing();

    [Decorator=withBackgroundScope]
    Sequence<i32> do_expensive_thing2();

    [Decorator=withErrorReporter, Throws=RustyError]
    bool do_crashy_thing();

    [Decorator=withErrorReporter, Throws=RustyError]
    u8 do_crashy_thing2();
}
```

### uniffi.toml

The decorator implementations are specified in `uniffi.toml`:

```toml

[bindings.kotlin.decorators.withBackgroundScope]
imports = [
    "kotlinx.coroutines.withContext",
    "myapp.dispachers.backgroundScope",
]
decorator = "withContext(backgroundScope)"
async = true

[bindings.kotlin.decorators.withErrorReporter]
imports = [
    "myapp.logging.errorReporter",
    "myapp.logging.withErrorReporter",
]
decorator = "withErrorReporter"
return_type = "{}?"
handles_exceptions = true
```

### Usage

The above code will make UniFFI generate the code needed to wrap the methods of `DemoRustObject`
with decorator functions.  This makes it behave exactly like the `WrappedDemoRustObject` class from
the problem statement without the hand-written boilerplate.

## Specification

### UDL

* Any function or method can be annotated with the `[Decorator={name}]` attribute

### uniffi.toml

* Add a table named `[bindings.{language}.decorators.{name}]` to wrap all functions/methods
  annotated with `[Decorator={name}]` in the generated code.  The following keys are supported:
  * `decorator`: expression for the decorator function that:
    * inputs a zero-argument closure that invokes the original method call.
    * arranges for that closure to be called somehow, not necessarily immediately.
    * can return any type it wants -- often decorators will transform the original return type.
    * for typed languages, is generic on the return value of the original method.
  * `imports` (optional): list of imports to add at the top of the file.  Use this to bring in
    dependencies needed for the decorator expression.
  * `return_type` (optional): return type for the decorated function.  The default is the
    same return type as the original function.
  * `async` (optional): If true, the decorated function will be defined as an
    async/suspend function
  * `handles_exceptions` (optional): if true, the decorated function will not define any exceptions
    thrown.  The default is throwing the same exceptions as the original function.

* If this table is not present, then UniFFI will not generate any special code for the decorated
  functions.  They will behave exactly as if the `[Decorated]` attribute was not present.
