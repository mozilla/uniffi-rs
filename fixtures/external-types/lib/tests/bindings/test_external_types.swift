/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import external_types_lib

// TODO: use an actual test runner.


// Test a round trip
do {
    let ct = getCombinedType(cval: CombinedType(
        cot: CrateOneType(sval: "test"),
        ctt: CrateTwoType(ival: 42)
    ))
    assert(ct.cot.sval == "test")
    assert(ct.ctt.ival == 42)
}

// Test passing in null value
do {
    let ct = getCombinedType(cval: nil)
    assert(ct.cot.sval == "hello")
    assert(ct.ctt.ival == 1)
}
