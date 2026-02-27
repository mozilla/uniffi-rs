/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Default arguments
assert(funcWithDefault() == "DEFAULT");
assert(funcWithDefault(arg: "NON-DEFAULT") == "NON-DEFAULT");

let complexMethods = ComplexMethods();
assert(complexMethods.methodWithDefault() == "DEFAULT")
assert(complexMethods.methodWithDefault(arg: "NON-DEFAULT") == "NON-DEFAULT")


// These just test that the argument names get mapped to camelCase
let _ = complexMethods.methodWithMultiWordArg(theArgument: "test")
let _ = funcWithMultiWordArg(theArgument: "test")

