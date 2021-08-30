// This is a helper for safely working with byte buffers returned from the Rust code.
// A rust-owned buffer is represented by its capacity, its current length, and a
// pointer to the underlying data.

@Structure.FieldOrder("capacity", "len", "data")
open class RustBuffer : Structure() {
    @JvmField var capacity: Int = 0
    @JvmField var len: Int = 0
    @JvmField var data: Pointer? = null

    class ByValue : RustBuffer(), Structure.ByValue
    class ByReference : RustBuffer(), Structure.ByReference

    companion object {
        internal fun alloc(size: Int = 0) = rustCall() { status ->
            _UniFFILib.INSTANCE.{{ ci.ffi_rustbuffer_alloc().name() }}(size, status)
        }

        internal fun free(buf: RustBuffer.ByValue) = rustCall() { status ->
            _UniFFILib.INSTANCE.{{ ci.ffi_rustbuffer_free().name() }}(buf, status)
        }

        internal fun reserve(buf: RustBuffer.ByValue, additional: Int) = rustCall() { status ->
            _UniFFILib.INSTANCE.{{ ci.ffi_rustbuffer_reserve().name() }}(buf, additional, status)
        }
    }

    @Suppress("TooGenericExceptionThrown")
    fun asByteBuffer() =
        this.data?.getByteBuffer(0, this.len.toLong())?.also {
            it.order(ByteOrder.BIG_ENDIAN)
        }
}

// This is a helper for safely passing byte references into the rust code.
// It's not actually used at the moment, because there aren't many things that you
// can take a direct pointer to in the JVM, and if we're going to copy something
// then we might as well copy it into a `RustBuffer`. But it's here for API
// completeness.

@Structure.FieldOrder("len", "data")
open class ForeignBytes : Structure() {
    @JvmField var len: Int = 0
    @JvmField var data: Pointer? = null

    class ByValue : ForeignBytes(), Structure.ByValue
}


// A helper for structured writing of data into a `RustBuffer`.
// This is very similar to `java.nio.ByteBuffer` but it knows how to grow
// the underlying `RustBuffer` on demand.
//
// TODO: we should benchmark writing things into a `RustBuffer` versus building
// up a bytearray and then copying it across.

class RustBufferBuilder() {
    var rbuf = RustBuffer.ByValue()
    var bbuf: ByteBuffer? = null

    init {
        val rbuf = RustBuffer.alloc(16) // Totally arbitrary initial size
        rbuf.writeField("len", 0)
        this.setRustBuffer(rbuf)
    }

    internal fun setRustBuffer(rbuf: RustBuffer.ByValue) {
        this.rbuf = rbuf
        this.bbuf = this.rbuf.data?.getByteBuffer(0, this.rbuf.capacity.toLong())?.also {
            it.order(ByteOrder.BIG_ENDIAN)
            it.position(rbuf.len)
        }
    }

    fun finalize() : RustBuffer.ByValue {
        val rbuf = this.rbuf
        // Ensure that the JVM-level field is written through to native memory
        // before turning the buffer, in case its recipient uses it in a context
        // JNA doesn't apply its automatic synchronization logic.
        rbuf.writeField("len", this.bbuf!!.position())
        this.setRustBuffer(RustBuffer.ByValue())
        return rbuf
    }

    fun discard() {
        val rbuf = this.finalize()
        RustBuffer.free(rbuf)
    }

    internal fun reserve(size: Int, write: (ByteBuffer) -> Unit) {
        // TODO: this will perform two checks to ensure we're not overflowing the buffer:
        // one here where we check if it needs to grow, and another when we call a write
        // method on the ByteBuffer. It might be cheaper to use exception-driven control-flow
        // here, trying the write and growing if it throws a `BufferOverflowException`.
        // Benchmarking needed.
        if (this.bbuf!!.position() + size > this.rbuf.capacity) {
            rbuf.writeField("len", this.bbuf!!.position())
            this.setRustBuffer(RustBuffer.reserve(this.rbuf, size))
        }
        write(this.bbuf!!)
    }

    fun putByte(v: Byte) {
        this.reserve(1) { bbuf ->
            bbuf.put(v)
        }
    }

    fun putShort(v: Short) {
        this.reserve(2) { bbuf ->
            bbuf.putShort(v)
        }
    }

    fun putInt(v: Int) {
        this.reserve(4) { bbuf ->
            bbuf.putInt(v)
        }
    }

    fun putLong(v: Long) {
        this.reserve(8) { bbuf ->
            bbuf.putLong(v)
        }
    }

    fun putFloat(v: Float) {
        this.reserve(4) { bbuf ->
            bbuf.putFloat(v)
        }
    }

    fun putDouble(v: Double) {
        this.reserve(8) { bbuf ->
            bbuf.putDouble(v)
        }
    }

    fun put(v: ByteArray) {
        this.reserve(v.size) { bbuf ->
            bbuf.put(v)
        }
    }
}

// Implement `lift()` by reading from a `RustBuffer`
internal inline fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): T {
    val buf = rbuf.asByteBuffer()!!
    try {
       val item = readItem(buf)
       if (buf.hasRemaining()) {
           throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
       }
       return item
    } finally {
        RustBuffer.free(rbuf)
    }
}

// Implement `lower()` by writing to a `RustBuffer`
internal inline fun<T> lowerIntoRustBuffer(v: T, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
    // TODO: maybe we can calculate some sort of initial size hint?
    val buf = RustBufferBuilder()
    try {
        writeItem(v, buf)
        return buf.finalize()
    } catch (e: Throwable) {
        buf.discard()
        throw e
    }
}

