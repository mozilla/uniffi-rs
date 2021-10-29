/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import wrapper_types

// TODO: use an actual test runner.

do {
    // Test simple values.
    let demo = getWrappedTypesDemo(demo: nil)
    assert(demo.json == "{\"demo\":\"string\"}")
    assert(demo.handle == 123)
}
