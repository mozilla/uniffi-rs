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

We'll handwrite a little decorator interface, then write an implementation that we pass the thread pool/coroutine context/whatever, and the error reporter.

```kotlin
interface DemoDecorator {
    fun onBackgroundThread(rustCall: () -> Unit)
    fun <T> withErrorReporter(rustCall: () -> T): T
    fun thenNotifyListeners(obj: DemoLib, rustCall: () -> T)
}

class MyDemoDecorator(
    val backgroundScope: CoroutineContext,
    val errorReporter: (e: Exception) -> Unit,
    val listener: () -> Void
) : DemoDecorator {
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

    fun thenNotifyListeners(obj: DemoLib, rustCall: () -> T) {
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
    val decorator: DemoDecorator = MyDemoDecorator()
) {
    val demoRustObject = new DemoRustObject()

    fun doExpensiveThing() = decorator.onBackgroundThread {
        demoRustObject.doExpensiveThing()
    }

    fun doCrashyThing() = decorator.withErrorReporter {
        demoRustObject.doCrashyThing()
    }

    fun doConsequentialThing() = decorator.thenNotifyListeners(this) {
        demoRustObject.doConsequentialThing()
    }
}
```

This is much better, but it's still looking a bit cut and pasty. Enter decorator objects.

## Solution

```webidl
namespace demo {}

[Decorator]
interface DemoDecorator {
    Any? with_error_reporter();
    void on_background_thread();
    Any  then_notify_listeners();
}

[Decorator=DemoDecorator]
interface DemoRustObject {
    [CallsWith=on_background_thread]
    void do_expensive_thing();

    [Throws=RustyError, CallsWith=with_error_reporter]
    void do_crashy_thing();

    [CallsWith=then_notify_listeners]
    void do_consequential_thing();
}
```

With this UDL: the `[Decorator]` annotation declares an `DemoDecorator` as a decorator object. Decorator objects never cross the FFI.

> Decorator objects take their name from [Python's decorator methods][py-decorators].
>
> In Pythonic terms, Uniffi's decorator objects are a collection of decorator functions.
>
> Swift and Kotlin don't provide functionality to capture arbitrary `*args` and call a function with those same `*args`, so
> at this time, decorator objects aren't as powerful as Python decorators. Nevertheless, they can be still quite useful.

[py-decorators]: https://www.python.org/dev/peps/pep-0318/#on-the-name-decorator

They are implemented as a `protocol` in Swift, and `interface` in Kotlin.

They declare zero argument methods, but can return an arbitrary concrete type, `void` or the generic `Any` type. The `DemoDecorator` interface is now generated,
and each method accepts the decorated object and a generic closure which will call the Rust code.

```kotlin
// generated by uniffi
interface DemoDecorator<ObjectType> {
    fun <ReturnType> onBackgroundThread(obj: ObjectType, rustCall: () -> ReturnType)
    fun <ReturnType> withErrorReporter(obj: ObjectType, rustCall: () -> ReturnType): ReturnType?
    fun <ReturnType> thenNotifyListeners(obj: ObjectType, rustCall: () -> ReturnType)
}
```

The `[Decorator=DemoDecorator]` annotation above the `DemoRustObject` `interface` declaration ties the decorator object to our original `DemoRustObject` class.

This changes the generated API of `DemoRustObject`:

* it adds a `DemoDecorator` argument to every constructor.
* it changes the return types and throws types for each method called with a decorator method to match the decorator method. Where the decorator method returns `Any`, the original return type remains.

Now the generated Rust calls go through the app-specific decorator methods, we could more safely hand the rust object to the application to use directly.

```kotlin
val decorator = MyDemoDecorator(backgroundScope, errorReporter, listener)
val demoRustObject = DemoRustObject(decorator)
```

This is a considerable improvement! Now the decorator methods can be specified by the app, and the UDL uses them. Each time the UDL changes, the foreign language bindings keeps up.

### Re-writing the wrapper

In our example above, we had a handwritten `DemoLib` which had all the methods of the `DemoRustObject` but did all error catching and launching on a background thread. It was also a good place to put additional handwritten code. With decorator objects, the need for `DemoLib` all but disappears.

However, there may already be a considerable amount of application code pointing to it, which we don't want to change.

Uniffi generates a Kotlin `interface` or Swift `protocol` for each `Object` in the UDL. In this case, `DemoRustObject` is the class, and `DemoRustObjectInterface` is the interface.

#### Kotlin

We can use this and [Kotlin's Interface delegation features][1] to re-write `DemoLib` so that it too keeps up and in sync with the UDL.

[1]: https://kotlinlang.org/docs/delegation.html

```kotlin
class DemoLib private constructor(
    demoRustObject: DemoRustObject
) : DemoRustObjectInterface by demoRustObject {

    constructor(decorator: DemoDecorator) = this(DemoRustObject(decorator))

    constructor(
        backgroundScope: CoroutineContext,
        errorReporter: (e: Exception) -> Unit,
        listener: () -> Unit
    ) = this(MyDemoDecorator(backgroundScope, errorReporter, listener))
}
```

We can then add hand-written code to this class.

#### Swift

In Swift, we can't get `DemoLib` to implement `DemoRustObjectProtocol`, and automatically implement the methods with `DemoRustObject`. Instead we literally replace it with a `typealias` and add a convenience constructor to take the original arguments. Handwritten code can then be added to this extension.

```swift
typealias DemoLib = DemoRustObject
extension DemoLib {
    convenience init(
        backgroundQueue: OperationQueue,
        errorReporter: @escaping (Error) -> Void,
        listener: @escaping () -> Void
    ) {
        self.init(
            MyDemoDecorator(
                backgroundQueue: backgroundQueue,
                errorReporter: errorReporter,
                listener: listener
            )
        )
    }
}
```
