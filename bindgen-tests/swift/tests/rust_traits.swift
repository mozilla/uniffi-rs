/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

assert(RustTraitTest(a: 1, b: 2).debugDescription == "debug-test-string")
assert(RustTraitTest(a: 1, b: 2).description == "display-test-string")

// The Rust code only uses `a` for the equality
assert(RustTraitTest(a: 1, b: 2) == RustTraitTest(a: 1, b: 3))
assert(RustTraitTest(a: 1, b: 2) != RustTraitTest(a: 2, b: 2))

// The Rust code only uses `a` for the ordering
assert(RustTraitTest(a: 1, b: 2) < RustTraitTest(a: 2, b: 3))
assert(RustTraitTest(a: 1, b: 2) <= RustTraitTest(a: 1, b: 3)
    && RustTraitTest(a: 1, b: 2) >= RustTraitTest(a: 1, b: 3))

// The Rust code only uses `a` for the hash
assert(RustTraitTest(a: 1, b: 2).hashValue == RustTraitTest(a: 1, b: 3).hashValue)
assert(RustTraitTest(a: 2, b: 2).hashValue != RustTraitTest(a: 1, b: 2).hashValue)
