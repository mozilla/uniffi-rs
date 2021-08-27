/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.decorators.*
import my.decorator.MyDecorator

// Create an instance of an interface
val rustObj = RustObject()
// Create a decorator for the interface
val decorator = MyDecorator()
// Create a decorated version of it
val decorated = DecoratedRustObject(rustObj, decorator)
// Decorate an object created with the secondary constructor
val string1 = "placeholder string"
val decorated2 = DecoratedRustObject(RustObject.fromString(string1), decorator)

assert(decorated.length() == 0) { "generic return" }
assert(decorated2.length() == string1.length) { "generic return" }

assert(decorated.getString() == 1) { "different return type from method's own" }
assert(decorated.getString() == 2) { "code is run each time the method is run" }
assert(decorated2.getString() == 3) { "decorator is shared between objects" }

val string2 = "meta-syntactic variable values"
assert(decorated.identityString(string2) == Unit) { "void return" }
assert(decorator.lastString == string2) { "Decorator is actually called" }
