import uniffi.uniffi_bindgen_tests.*
import java.nio.ByteBuffer

var bytes = ByteArray(4)
bytes[0] = 0.toByte()
bytes[1] = 1.toByte()
bytes[2] = 2.toByte()
bytes[3] = 3.toByte()
assert(roundtripBytes(bytes).contentEquals(bytes))

// Zero-copy &[u8] — proc-macro path
fun directBufferOf(vararg bytes: Byte): ByteBuffer {
    val buf = ByteBuffer.allocateDirect(bytes.size)
    buf.put(bytes)
    buf.flip()
    return buf
}

assert(sumBytesProcmacro(ByteBuffer.allocateDirect(0)) == 0u)
assert(sumBytesProcmacro(directBufferOf(1, 2, 3)) == 6u)
assert(firstByteProcmacro(ByteBuffer.allocateDirect(0)) == null)
assert(firstByteProcmacro(directBufferOf(42)) == 42.toUByte())
