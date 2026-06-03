/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(roundtripOptionU8(a: 67) == 67);
assert(roundtripOptionU8(a: nil) == nil);
assert(roundtripOptionI8(a: 67) == 67);
assert(roundtripOptionI8(a: nil) == nil);
assert(roundtripOptionU16(a: 67) == 67);
assert(roundtripOptionU16(a: nil) == nil);
assert(roundtripOptionI16(a: 67) == 67);
assert(roundtripOptionI16(a: nil) == nil);
assert(roundtripOptionU32(a: 67) == 67);
assert(roundtripOptionU32(a: nil) == nil);
assert(roundtripOptionI32(a: 67) == 67);
assert(roundtripOptionI32(a: nil) == nil);
assert(roundtripOptionU64(a: 67) == 67);
assert(roundtripOptionU64(a: nil) == nil);
assert(roundtripOptionI64(a: 67) == 67);
assert(roundtripOptionI64(a: nil) == nil);
assert(roundtripOptionF32(a: 67.0) == 67.0);
assert(roundtripOptionF32(a: nil) == nil);
assert(roundtripOptionF64(a: 67.0) == 67.0);
assert(roundtripOptionF64(a: nil) == nil);
assert(roundtripOptionBool(a: true) == true);
assert(roundtripOptionBool(a: nil) == nil);
assert(roundtripOptionString(a: "test-string") == "test-string");
assert(roundtripOptionString(a: nil) == nil);
assert(roundtripOptionRec(a: OptionsRec(a: 67)) == OptionsRec(a: 67))
assert(roundtripOptionRec(a: nil) == nil);
