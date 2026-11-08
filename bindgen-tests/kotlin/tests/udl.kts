/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.uniffi_bindgen_tests.*

assert(udlSumFields(UdlType(0.5, 1.5)) == 2.0)
assert(udlMakeType(0.5, 1.5) == UdlType(0.5, 1.5))
assert(udlRemoteSumFields(UdlRemoteType(0.5, 1.5)) == 2.0)
