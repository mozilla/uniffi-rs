/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import imported_types_lib
import Foundation

let ct = getCombinedType(value: nil)
assert(ct.uot.sval == "hello")
assert(ct.guid ==  "a-guid")
assert(ct.url ==  URL(string: "http://example.com/"))

let ct2 = getCombinedType(value: ct)
assert(ct == ct2)
