/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import uniffi.fixture.mut_bytes.*
import java.nio.ByteBuffer

fun directBufferOf(vararg bytes: Byte): ByteBuffer {
    val buf = ByteBuffer.allocateDirect(bytes.size)
    buf.put(bytes)
    buf.flip()
    return buf
}

// Zero-copy &mut [u8] via UDL [ByMutRef]. Rust writes land in the direct buffer.
val fillMe = ByteBuffer.allocateDirect(4)
fillBytesUdl(fillMe)
assert(fillMe.get(0) == 0.toByte())
assert(fillMe.get(1) == 1.toByte())
assert(fillMe.get(2) == 2.toByte())
assert(fillMe.get(3) == 3.toByte())

val incMe = directBufferOf(1, 2, 3)
incrementBytesUdl(incMe)
assert(incMe.get(0) == 2.toByte())
assert(incMe.get(1) == 3.toByte())
assert(incMe.get(2) == 4.toByte())

// Empty buffer must not crash.
val empty = ByteBuffer.allocateDirect(0)
fillBytesUdl(empty)
