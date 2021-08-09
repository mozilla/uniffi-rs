/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import java.util.concurrent.*

import uniffi.external_types_lib.*

// TODO: use an actual test runner.

val ct = getCombinedType(CombinedType(
    CrateOneType("test"),
    CrateTwoType(42),
));
assert(ct.cot.sval == "test");
assert(ct.ctt.ival == 42);

val ct2 = getCombinedType(null);
assert(ct2.cot.sval == "hello");
assert(ct2.ctt.ival == 1);
