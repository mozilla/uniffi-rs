/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Test calling and returning a single argument
assert(roundtripU8(a: 42) == 42);
assert(roundtripI8(a: -42) == -42);
assert(roundtripU16(a: 42) == 42);
assert(roundtripI16(a: -42) == -42);
assert(roundtripU32(a: 42) == 42);
assert(roundtripI32(a: -42) == -42);
assert(roundtripU64(a: 42) == 42);
assert(roundtripI64(a: -42) == -42);
assert(roundtripF32(a: 0.5) == 0.5);
assert(roundtripF64(a: -3.5) == -3.5);
assert(roundtripBool(a: true) == true);
assert(roundtripString(a: "ABC") == "ABC");
// Test calling a function with lots of args
// This function will sum up all the numbers, then negate the value since we passed in `true`
assert(sumWithManyTypes(a: 1, b: -2, c: 3, d: -4, e: 5, f: -6, g: 7, h: -8, i: 9.5, j: -10.5, negate: true) == 5);
