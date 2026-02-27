/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripOption(a: 42) == 42);
assert(roundtripOption(a: nil) == nil);
assert(roundtripVec(a: [1, 2, 3]) == [1, 2, 3]);
assert(roundtripHashMap(a: ["a": 1, "b": 2]) == ["a": 1, "b": 2])
assert(roundtripComplexCompound(a: [
    ["a": 1, "b": 2]
]) == [
    ["a": 1, "b": 2]
])
assert(roundtripComplexCompound(a: nil) == nil);
