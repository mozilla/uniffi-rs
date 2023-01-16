/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.imported_types_lib.*

val ct = getCombinedType(null)
assert(ct.uot.sval == "hello")
assert(ct.guid ==  "a-guid")
assert(ct.url ==  java.net.URL("http://example.com/"))

val ct2 = getCombinedType(ct)
assert(ct == ct2)

assert(getUrl(java.net.URL("http://example.com/")) ==  java.net.URL("http://example.com/"))
