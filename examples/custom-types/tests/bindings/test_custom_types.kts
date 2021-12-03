/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*

import uniffi.wrapper_types.*
import uniffi.custom_types.*

// TODO: use an actual test runner.

// Get the custom types and check their data
val demo = getCustomTypesDemo(null)
assert(demo.url == "http://example.com/")
assert(demo.handle == 123L)

// Change some data and ensure that the round-trip works
demo.url = "http://new.example.com/"
demo.handle = 456;
assert(demo == getCustomTypesDemo(demo))
