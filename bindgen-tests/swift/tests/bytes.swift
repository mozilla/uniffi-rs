/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi_bindgen_tests
import Foundation

var data = Data.init()
data.append(contentsOf: [1, 2, 3, 4])
assert(roundtripBytes(a: data) == data);

// Zero-copy &[u8] — proc-macro path
assert(sumBytesProcmacro(buf: Data()) == 0)
assert(sumBytesProcmacro(buf: Data([1, 2, 3])) == 6)
assert(firstByteProcmacro(buf: Data()) == nil)
assert(firstByteProcmacro(buf: Data([42])) == 42)

// Zero-copy &mut [u8] — proc-macro path. Rust writes land in the caller's Data.
var fillMe = Data(count: 4)
fillBytesProcmacro(buf: &fillMe)
assert(fillMe == Data([0, 1, 2, 3]))

var incMe = Data([1, 2, 3])
incrementBytesProcmacro(buf: &incMe)
assert(incMe == Data([2, 3, 4]))

// Empty buffer must not crash.
var empty = Data()
fillBytesProcmacro(buf: &empty)
assert(empty == Data())
