/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import Foundation
import zero_copy_bytes

// UDL path — proc-macro path is covered in bindgen-tests.
assert(sumBytesUdl(buf: Data()) == 0)
assert(sumBytesUdl(buf: Data([10, 20])) == 30)
