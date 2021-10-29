/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*

import uniffi.wrapper_types.*

// TODO: use an actual test runner.

val demo = getWrappedTypesDemo(null)
assert(demo.json == """{"demo":"string"}""")
assert(demo.handle == 123L)
demo.handle = 456;

val demo2 = getWrappedTypesDemo(demo)
assert(demo2.handle == 456L)
