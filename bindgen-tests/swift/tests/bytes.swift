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
