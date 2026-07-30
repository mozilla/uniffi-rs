/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests
import Foundation

var data = Data.init()
data.append(contentsOf: [1, 2, 3, 4])
assert(roundtripBytes(a: data) == data);

// Zero-copy &[u8]
assert(sumBytes(buf: Data()) == 0)
assert(sumBytes(buf: Data([1, 2, 3])) == 6)
assert(firstByte(buf: Data()) == nil)
assert(firstByte(buf: Data([42])) == 42)

// Zero-copy &mut [u8]. Rust writes land in the caller's Data.
var fillMe = Data(count: 4)
fillBytes(buf: &fillMe)
assert(fillMe == Data([0, 1, 2, 3]))

var incMe = Data([1, 2, 3])
incrementBytes(buf: &incMe)
assert(incMe == Data([2, 3, 4]))

// Empty buffer must not crash.
var empty = Data()
fillBytes(buf: &empty)
assert(empty == Data())
