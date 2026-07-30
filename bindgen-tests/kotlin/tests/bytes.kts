import uniffi.uniffi_bindgen_tests.*
import java.nio.ByteBuffer

var bytes = ByteArray(4)
bytes[0] = 0.toByte()
bytes[1] = 1.toByte()
bytes[2] = 2.toByte()
bytes[3] = 3.toByte()
assert(roundtripBytes(bytes).contentEquals(bytes))

// Zero-copy &[u8]
fun directBufferOf(vararg bytes: Byte): ByteBuffer {
    val buf = ByteBuffer.allocateDirect(bytes.size)
    buf.put(bytes)
    buf.flip()
    return buf
}

assert(sumBytes(ByteBuffer.allocateDirect(0)) == 0u)
assert(sumBytes(directBufferOf(1, 2, 3)) == 6u)
assert(firstByte(ByteBuffer.allocateDirect(0)) == null)
assert(firstByte(directBufferOf(42)) == 42.toUByte())

// Zero-copy &mut [u8]. Rust writes land in the direct buffer.
val fillMe = ByteBuffer.allocateDirect(4)
fillBytes(fillMe)
assert(fillMe.get(0) == 0.toByte())
assert(fillMe.get(1) == 1.toByte())
assert(fillMe.get(2) == 2.toByte())
assert(fillMe.get(3) == 3.toByte())

val incMe = directBufferOf(1, 2, 3)
incrementBytes(incMe)
assert(incMe.get(0) == 2.toByte())
assert(incMe.get(1) == 3.toByte())
assert(incMe.get(2) == 4.toByte())

// Empty buffer must not crash.
val empty = ByteBuffer.allocateDirect(0)
fillBytes(empty)
