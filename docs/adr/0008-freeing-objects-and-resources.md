# Freeing objects and resources

* Status: Accepted
* Deciders: Uniffi developers, as approved in PR-1787
* Date: 2023-10-18

Previous discussion: [PR 1394](https://github.com/mozilla/uniffi-rs/issues/1394), [Issue 8](https://github.com/mozilla/uniffi-rs/issues/8)

## Context and Problem Statement

UniFFI objects always hold a reference that needs to be released and sometimes hold a resource that needs to be closed.
The current design conflates the two issues.
We can address both concerns better and improve ergonomics by dealing with the two separately.

This ADR is written in language-neutral terms so that the issue can be discussed in general.
However it's most clear in the Kotlin implementation and this ADR will affect that code the most, see Appendix 1 for the proposed changes.

## Releasing References
When a Rust object is passed to the foreign side, the foreign code stores a reference to a Rust `Arc<>`.
Conversely, foreign objects passed into Rust (eg, callbacks or trait implementations) cause Rust to hold a reference to the foreign object.

UniFFI needs to ensure that when the generated objects are destroyed, the reference on the other side of the FFI is released.
It is acceptable for this release to be delayed and/or non-deterministic as long as it eventually happens.
In particular, it is acceptable for it to be handled by garbage collection.

## Closing Resources

When an object holds a significant resource (eg, some scarce operating-system owned resource), it may not be acceptable to couple the resource to the lifetime of the foreign object.
In this case, it MUST be to possible close the resource while the foreign object is still alive and before its reference is released.

These same considerations apply in both directions across the FFI.
If a garbage-collected foreign object stores a resource, then there MUST be a way to close the resource before the garbage collection happens.
If a Rust object holds a resource, there MUST be a way to close that resource even if there may be other `Arc<>` references.

## Object containers

The discussion above has focused on individual objects, but in any non-trivial application many objects interact.
For example, consider a simple `Enum` or `dictionary` - in the usual case, these objects hold references to a non-resource holding objects.
Given the discussion above, it's acceptable for these references to be released using the standard memory management of the foreign language.

However, the `Enum`, `dictionary`, `Vec`, `HashMap`, etc. may contain an object that itself contains a resource.
While the component author is able to offer methods to close the resource on the objects themselves, they are not able to introduce methods to all the container objects which might hold the object.
So the question here is whether UniFFI itself should try and offer such capabilities.

## Foreign "sugar" for closing resources.

Objects which hold significant resources offer the capability to closing those resources even while the object remains alive.
For example, most foreign languages which expose a "file" object tend to provide a `close()` method that ensures the handle is released to the operating system.

For almost all languages it is possible use something like a `try/finally` pattern to ensure the object is closed as soon as practical.
This pattern is so common that some languages offer syntactic sugar for this express purpose.
For example, Python offers a "context manager", Kotlin offers the `use` extension for objects that implement `Closable`, etc.

Ideally, UniFFI would offer a way so consumers can opt-in to implementing this capability.
This must be opt-in because that opt-in process must tell UniFFI exactly how the resource should be closed - while it will be very common for this to be via a `close()` method, UniFFI can't assume this (eg, a Python context manager doesn't insist on any particular technique)

## Decision Drivers

* UniFFI should present simple and clear abstractions to users.

* UniFFI should let users explicitly decide how they want to deal with resources.
  There is no way UniFFI can automatically differentiate between objects that hold significant resources versus those that do not.
  Thus, components must "opt in" somehow to this facility, where the opt-in could be anywhere from fully manual closing (eg, by supplying a close method and documenting it must be called ASAP) or something more formal (ie, telling UniFFI this is such an object and UniFFI supplying some builtin support for this concept)

* There is nothing UniFFI can do by itself to ensure that Rust references are released when foreign objects are destroyed.
  This responsibility must fall on the foreign language in one way or another.

* It seems unreasonable for UniFFI to be expected to track object containers and propagate any close operations as this would quickly become an ergonomic and technical quagmire (eg, would be give it to all objects just in-case such an object might be added in the future?
  If not, how would the user (or even UniFFI itself) know whether the container actually held such an object? etc)

## Considered Options

### [Option 1] UniFFI handles releasing references, closing resources is largely left up to the user

* Rely on all languages having *some* strategy for automatically closing resources as they are no longer reachable.
  For most languages this will be something like a destructor, or some other facility with destructor-like semantics.
  If a language has no such capability, then the bindings will be forced to implement some special facility for each and every object to be explicitly freed, but this strategy is unlikely to be useful in practice.

* Don't automatically implement any methods to close a resource; if an object has resources that should be closed while the object remains alive, the component author should supply a mechanism to do that.

* Investigate providing a facility for objects to opt-in to "foreign sugar" for closing the resources.
  The exact mechanics for this are beyond the scope of the ADR.

* Offer some general advice summarizing this in the documents, like "define a close() method and document it", but otherwise leave this problem for users to solve.

* Don't do anything special for object containers.
  If the field of a record contains a resource, then it's the user's responsibility to properly close that field.

### [Option 2] Resource and Object as separate types

* Require UDL users to label their interfaces as either a `Resource` or `Object` and proc-macro users to derive either `Resource` or `Object`.
* `Object` instances are assumed to not contain significant resources and we rely on the foreign memory management for releasing the reference.
* `Resource` instances would have a formal mechanism for closing their resource and releasing their reference at the same time.
  Optionally, we may also use the foreign memory management as a fallback to close the resource and release the reference if the user did not do so explicitly.

### [Option 3] Make Object do both (ie, assume all objects should get close methods)

* Like option 2, but treat all objects as though they have significant resources to be freed.

## Pros and Cons of the Options

### [Option 1] UniFFI handles releasing references, closing resources is largely left up to the user

* Good, because gives the user control over their object namespace.  UniFFI doesn't define any methods or assume any methods exist, etc
* Bad, because it relies on a contractual agreement to ensure resource leaks are avoided and doesn't help if those obligations aren't met.
* Bad, because it requires the "closeable" object to deal with some extra state - for example, if an object provides a `close()` method
  which may be called before the object is destroyed, the object must deal with (a) the possibility that the object continues to be used
  in some capacity after this close and (b) that as the object is destroyed `close()` may or may not have been called.
* Good, because the same general solution works for when Rust is holding a foreign reference.
* Bad, because generating "foreign sugar" means each object must opt-in to this behaviour instead of getting it for free.
* Good, because making resource closing the user's responsibility extends to object containers as well.
  If a library authors decides that it makes sense to define an enum variant that contains some resource-holding object, then it's the library consumer's responsibility to ensure that resource is properly closed.

### [Option 2] Resource and Object as separate types

* Good, because users explicitly decide on their interface behavior.
* Good, because it means we could infer the "foreign sugar" situation and automatically implement special interfaces like `AutoCloseable`
  in Kotlin for a Context Manager in Python.
* Bad, because it means all objects must use the same signature for closing objects, whereas, eg, a connection to a database
  object might prefer the method to be called `close_connection` and might even allow parameters.
* Bad, because it implicitly defines some methods (eg, `close()`), preventing users from defining methods with those names that have different semantics.
* Bad, because it doesn't offer a reasonable solution for when Rust is holding a foreign reference.
* Good, because the distinction makes deciding how to handle object containers easier.
  For example, enums can contain `Object` fields, but not `Resource` fields.

### [Option 3] Make Object do both (ie, assume all objects should get close methods)

* Good, because it's the easiest to implement.
* Good, because we can easily continue to implement special traits like `AutoCloseable`.
* Bad, for all the reasons Option 1 is bad.
* Bad, because it's not clear how to handle object containers in all cases.  Should enums be allowed to contain objects?

## Decision Outcome

Chosen option: **[Option 1] UniFFI handles releasing references, closing resources is largely left up to the user**

# Appendix 1 - what does this mean for our foreign bindings?

The above is all quite abstract - what does this mean in practice for our language bindings:

## Python

* Python bindings are already hooked up to `__del__` so nothing changes here.
* Once we have the "Foreign Sugar" opt-in mechanism in place we will generate context managers for such objects.

## Swift

* Existing `deinit` implementation is suitable so nothing changes here.

## Kotlin

* UniFFI will implement a mechanism that automatically frees Rust references when the corresponding foreign object is destroyed.
  [Effective Java, 3rd edition](https://www.oreilly.com/library/view/effective-java-3rd/9780134686097/) recommends never using destructors.
  It recommends avoiding the [Cleaner API](https://docs.oracle.com/javase%2F9%2Fdocs%2Fapi%2F%2F/java/lang/ref/Cleaner.html), but does say that it may be appropriate for freeing a "native peer", like the UniFFI reference.
  The Android docs recommend using the [ReferenceQueue](https://www.android-doc.com/reference/java/lang/ref/ReferenceQueue.html) API, which is the low-level class that `Cleaner` uses (https://www.android-doc.com/reference/java/lang/Object.html#finalize())

* We will no longer generate the `Closable` or `AutoClosable` interface for objects.
* Once we have the "Foreign Sugar" opt-in mechanism in place we will generate the `Closable` interface for all such objects (maybe `AutoClosable` as well although that is mostly only useful for Java interoperability).
