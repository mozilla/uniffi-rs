/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Records
let r = RecWithDefault()
assert(r.n == 42)
assert(r.v == [])

// Enums
let e = EnumWithDefault.otherVariant()
assert(e == EnumWithDefault.otherVariant(a: "default"))

// Default arguments
assert(funcWithDefault() == "DEFAULT");
assert(funcWithDefault(arg: "NON-DEFAULT") == "NON-DEFAULT");

let i = InterfaceWithDefaults();
assert(i.methodWithDefault() == "DEFAULT")
assert(i.methodWithDefault(arg: "NON-DEFAULT") == "NON-DEFAULT")

