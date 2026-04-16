/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Test passing values to Rust
inputU8(a: 42)
inputI8(a: -42)
inputU16(a: 42)
inputI16(a: -42)
inputU32(a: 42)
inputI32(a: -42)
inputU64(a: 42)
inputI64(a: -42)
inputF32(a: 0.5)
inputF64(a: -3.5)
inputBool(a: true)
inputString(a: "ABC")
// Test returning values to Swift
assert(outputU8() == 1);
assert(outputI8() == 1);
assert(outputU16() == 1);
assert(outputI16() == 1);
assert(outputU32() == 1);
assert(outputI32() == 1);
assert(outputU64() == 1);
assert(outputI64() == 1);
assert(outputF32() == 1.0);
assert(outputF64() == 1.0);
assert(outputBool() == true);
assert(outputString() == "test-string");
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
