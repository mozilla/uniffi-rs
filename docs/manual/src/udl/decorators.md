# Decorator objects

We'll be considering the UDL for an object called `DemoRustObject`:

```webidl
interface DemoRustObject {
    void do_expensive_thing();
    [Throws=RustyError]
    void do_crashy_thing();
    void do_consequential_thing();
}
```

`DemoRustObject` is a `struct` written in Rust, and uniffi generates a class of that name in the foreign languages which forward method calls to the Rust implementation.

## Problem

Just by the names of these methods, we can see that the application might not want to deal with the `DemoRustObject` by itself, without some proper care taken while calling its methods:

* it may want to run `do_expensive_thing()` off the main thread
* it may want to catch and report errors from `do_crashy_thing()`.
* it may want to inform the rest of the application each time `do_consequential_thing()` is called.

A common pattern when using uniffied code is to run some common, but app-specific, code before or after calling the Rust code.

```kotlin
class DemoLib(
    val backgroundScope: CoroutineContext,
    val errorReporter: (e: Exception) -> Unit,
    val listener: () -> Void
) {
    val demoRustObject = new DemoRustObject()

    fun doExpensiveThing() {
        backgroundScope.launch {
            demoRustObject.doExpensiveThing()
        }
    }

    fun doCrashyThing() {
        try {
            demoRustObject.doCrashyThing()
        } catch (e: Exception) {
            errorReporter(e)
        }
    }

    fun doConsequentialThing() {
        demoRustObject.doConsequentialThing()
        listener()
    }
}
```

This causes a proliferation of boiler plate code. Worse, everytime we add a new method to the `DemoRustObject`, handwritten foreign language code needs to be written to expose it safely.

The more methods that want to do similar things, the more repetitive this gets.

We could isolate the repeated code into a decorator class with `onBackgroundThread`, `withErrorReporter` and `thenNotifyListeners` methods.

```kotlin
class MyDemoDecorator(
    val backgroundScope: CoroutineContext,
    val errorReporter: (e: Exception) -> Unit,
    val listener: () -> Void
) {
    fun onBackgroundThread(rustCall: () -> Unit) {
        backgroundScope.launch {
            rustCall()
        }
    }

    fun <T> withErrorReporter(rustCall: () -> T) =
        try {
            rustCall()
        } catch (e: Exception) {
            errorReporter(e)
        }

    fun thenNotifyListeners(rustCall: () -> T) {
        try {
            rustCall()
        } finally {
            listener()
        }
    }
}
```

Then, we can re-write the `DemoLib` as:

```kotlin
class DemoLib(
    val demoRustObject: DemoRustObject,
    val decorator: MyDemoDecorator,
) {
    fun doExpensiveThing() = decorator.onBackgroundThread {
        demoRustObject.doExpensiveThing()
    }

    fun doCrashyThing() = decorator.withErrorReporter {
        demoRustObject.doCrashyThing()
    }

    fun doConsequentialThing() = decorator.thenNotifyListeners() {
        demoRustObject.doConsequentialThing()
    }
}
```

This is much better, but it's still looking a bit cut and pasty. Enter decorator objects.

## Decorators to the rescue

> Decorator objects take their name from [Python's decorator methods][py-decorators].
>
> In Pythonic terms, Uniffi's decorator objects are a collection of decorator functions.
>
> Swift and Kotlin don't provide functionality to capture arbitrary `*args` and call a function with those same `*args`, so
> at this time, decorator objects aren't as powerful as Python decorators. Nevertheless, they can be still quite useful.

[py-decorators]: https://www.python.org/dev/peps/pep-0318/#on-the-name-decorator

### UDL

In the UDL we:

* Specify that `DemoLib` should be a decorated version of `DemoRustObject` using the `[Decorated=`] annotation
* Specify how methods should by decorated using the `[CallsWith=]` annotation
* Don't specify the class that holds the decorator functions.  This is done on a per-app basis in `uniffi.toml`.

```webidl
namespace demo {}

[Decorated=DemoLib]
interface DemoRustObject {
    [CallsWith=onBackgroundThread]
    void do_expensive_thing();

    [Throws=RustyError, CallsWith=withErrorReporter]
    void do_crashy_thing();

    [CallsWith=thenNotifyListeners]
    void do_consequential_thing();
}
```

### Write your decorator class

```kotlin
package my.uniffi.bindings.package

class MyDemoDecorator(
    val backgroundScope: CoroutineContext,
    val errorReporter: (e: Exception) -> Unit,
    val listener: () -> Void
) {
    fun <T> onBackgroundThread(rustCall: () -> T) {
        backgroundScope.launch {
            rustCall()
        }
    }

    fun <T> withErrorReporter(rustCall: () -> T) =
        try {
            rustCall()
        } catch (e: Exception) {
            errorReporter(e)
        }

    fun thenNotifyListeners(rustCall: () -> T) {
        try {
            rustCall()
        } finally {
            listener()
        }
    }
}
```

### Specify the decorator class in your uniffi.toml

```toml

[bindings.kotlin.decorators.DemoRustObject]
class_name = MyDemoDecorator
```

### Use the decorator

The above code will make UniFFI define a `DemoLib` class that uses
`MyDemoDecorator` to decorate calls to `DemoRustObject`.  The `DemoLib`
constructor will input a `DemoRustObject` and `MyDemoDecorator`.

For example:

```kotlin
val demoLib = DemoLib(
   DemoRustObject(),
   MyDemoDecorator(backgroundScope, errorReporter, listener))

// This will call MyDemoDecorator.stateChange { DemoRustObject.doConsequentialThing() }
demoLib.doConsequentialThing()
```

This is a considerable improvement! Now the decorator methods can be specified by the app, and the UDL uses them. Each time the UDL changes, the foreign language bindings keeps up.

### Re-writing the wrapper

This would happen differently than the original proposal, but I'm not sure
exactly what to suggest because I don't understand the issue so well.  If the
point is to avoid changes to the code that constructs DemoLib/DemoRustObject,
then what about one of these?

  - Making DemoRustObject an open class and DemoLib a subclass whose constructor inputs the args you want
  - A factory function that inputs the args you want and returns a DemoRustObject

## Specification

### UDL

* Specify that an interface can be decorated by using the `[Decorated={class_name}]` attribute
  * `{class_name}` specifies that UniFFI should define a class named `class_name` where methods of the interface are called through decorators
* Specify how each method will be decorated in the decorator class using the `[CallsWith={decorator_name}]` attribute
  * This specifies that this method should be called with function named `decorator_name`
  * Methods that don't have the `CallsWith` attribute will be called directly

### uniffi.toml

* To enable a decorated class for a specific bindings language, add a table named `[bindings.{language}.decorators.{interface_name}]` with the following keys:
  * `class_name`: name of the decorator implementation class
  * `import` (optional): import line to add at the top of the file.  Use this to import your decorator class if needed.
  * `return_types` (swift only): Table that maps decorator names to the return type for the decorator function.
* If this table is not present, then UniFFI will not generate the decoratored class definition.

### Decorator classes

* Decorator classes must have a method corresponding to each `CallsWith` name.  This method will be used to decorate the interface's method.
  * It inputs a zero-argument closure that invokes the original method call.
  * It should arrange for that closure to be called somehow, not necessarily immediately.
  * It can return any type it wants -- often decoraters will transform the original return type.
  * For typed languages, it must be generic on the return value of the original method.
