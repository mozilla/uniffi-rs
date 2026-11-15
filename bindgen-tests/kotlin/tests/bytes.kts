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

// Rust sees exactly the bytes in [position, limit), not the start of the buffer.
val offsetBuf = directBufferOf(10, 20, 30, 40)
offsetBuf.position(1)
assert(firstByte(offsetBuf) == 20.toUByte())
assert(sumBytes(offsetBuf) == 90u)
// The call borrows the buffer; it must not move position or limit.
assert(offsetBuf.position() == 1)
assert(offsetBuf.limit() == 4)

val windowBuf = directBufferOf(10, 20, 30, 40)
windowBuf.position(1)
windowBuf.limit(3)
assert(sumBytes(windowBuf) == 50u)
assert(sumBytes(windowBuf.slice()) == 50u)

// position == limit on a non-empty buffer is an empty slice.
val drainedBuf = directBufferOf(10, 20, 30, 40)
drainedBuf.position(4)
assert(firstByte(drainedBuf) == null)
assert(sumBytes(drainedBuf) == 0u)

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

// Rust writes land in [position, limit); bytes outside it stay untouched.
fun allBytes(buf: ByteBuffer): List<Byte> {
    val view = buf.duplicate()
    view.clear()
    return List(view.capacity()) { view.get(it) }
}

val fillWindow = directBufferOf(9, 9, 9, 9, 9)
fillWindow.position(2)
fillWindow.limit(4)
fillBytes(fillWindow)
assert(allBytes(fillWindow) == listOf<Byte>(9, 9, 0, 1, 9))

val incWindow = directBufferOf(9, 9, 9, 9, 9)
incWindow.position(1)
incrementBytes(incWindow)
assert(allBytes(incWindow) == listOf<Byte>(9, 10, 10, 10, 10))

// Empty buffer must not crash.
val empty = ByteBuffer.allocateDirect(0)
fillBytes(empty)
