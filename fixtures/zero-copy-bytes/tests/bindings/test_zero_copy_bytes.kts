/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.zero_copy_bytes.*
import java.nio.ByteBuffer

// UDL path — proc-macro path is covered in bindgen-tests.
fun directBufferOf(vararg bytes: Byte): ByteBuffer {
    val buf = ByteBuffer.allocateDirect(bytes.size)
    buf.put(bytes)
    buf.flip()
    return buf
}

assert(sumBytesUdl(ByteBuffer.allocateDirect(0)) == 0u) { "empty UDL buf should sum to 0" }
assert(sumBytesUdl(directBufferOf(10, 20)) == 30u) { "UDL path: 10+20 = 30" }
