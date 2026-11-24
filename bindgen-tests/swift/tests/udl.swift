/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests

// Records defined in UDL (`udl_derive`)
assert(udlSumFields(udlType: UdlType(a: 0.5, b: 1.5)) == 2.0)
assert(udlMakeType(a: 0.5, b: 1.5) == UdlType(a: 0.5, b: 1.5))
assert(udlRemoteSumFields(remoteType: UdlRemoteType(a: 0.5, b: 1.5)) == 2.0)
