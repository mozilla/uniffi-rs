/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_mut_bytes
import Foundation

// Zero-copy &mut [u8] via UDL [ByMutRef]. Rust writes land in the caller's Data.
var fillMe = Data(count: 4)
fillBytesUdl(buf: &fillMe)
assert(fillMe == Data([0, 1, 2, 3]))

var incMe = Data([1, 2, 3])
incrementBytesUdl(buf: &incMe)
assert(incMe == Data([2, 3, 4]))

// Empty buffer must not crash.
var empty = Data()
fillBytesUdl(buf: &empty)
assert(empty == Data())
