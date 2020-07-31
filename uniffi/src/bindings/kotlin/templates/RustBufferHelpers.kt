// Helpers for lifting primitive data types from a bytebuffer.

fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, liftFrom: (ByteBuffer) -> T): T {
    val buf = rbuf.asByteBuffer()!!
    try {
       val item = liftFrom(buf)
       if (buf.hasRemaining()) {
           throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
       }
       return item
    } finally {
        RustBuffer.free(rbuf)
    }
}

fun Boolean.Companion.lift(v: Byte): Boolean {
    return v.toInt() != 0
}

fun Boolean.Companion.liftFrom(buf: ByteBuffer): Boolean {
    return Boolean.lift(buf.get())
}

fun Byte.Companion.lift(v: Byte): Byte {
    return v
}

fun Byte.Companion.liftFrom(buf: ByteBuffer): Byte {
    return buf.get()
}

fun Int.Companion.lift(v: Int): Int {
    return v
}

fun Int.Companion.liftFrom(buf: ByteBuffer): Int {
    return buf.getInt()
}


fun Long.Companion.lift(v: Long): Long {
    return v
}

fun Long.Companion.liftFrom(buf: ByteBuffer): Long {
    return buf.getLong()
}

fun Float.Companion.lift(v: Float): Float {
    return v
}

fun Float.Companion.liftFrom(buf: ByteBuffer): Float {
    return buf.getFloat()
}

fun Double.Companion.lift(v: Double): Double {
    return v
}

fun Double.Companion.liftFrom(buf: ByteBuffer): Double {
    val v = buf.getDouble()
    return v
}

// I can't figure out how to make a generic implementation of (Any?).liftFrom, and IIUC there are some
// restrictions on generics in Kotlin (inherited from the JVM) that make it impossible to write in the
// style I want here. So, we use a standalone helper.

fun<T> liftOptional(rbuf: RustBuffer.ByValue, liftFrom: (ByteBuffer) -> T): T? {
    return liftFromRustBuffer(rbuf) { buf -> liftFromOptional(buf, liftFrom) }
}

fun<T> liftFromOptional(buf: ByteBuffer, liftFrom: (ByteBuffer) -> T): T? {
    if (! Boolean.liftFrom(buf)) {
        return null
    }
    return liftFrom(buf)
}

fun<T> liftSequence(rbuf: RustBuffer.ByValue, liftFrom: (ByteBuffer) -> T): List<T> {
    return liftFromRustBuffer(rbuf) { buf -> liftFromSequence(buf, liftFrom) }
}

fun<T> liftFromSequence(buf: ByteBuffer, liftFrom: (ByteBuffer) -> T): List<T> {
    val len = Int.liftFrom(buf)
    return List<T>(len) {
        liftFrom(buf)
    }
}

// Helpers for lowering primitive data types into a bytebuffer.
// Since we need to allocate buffers from rust, the lowering process needs to be
// able to calculate ahead-of-time what the required size of the buffer will be.

fun<T> lowerIntoRustBuffer(v: T, lowersIntoSize: (T) -> Int, lowerInto: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    val buf = RustBuffer.alloc(lowersIntoSize(v))
    try {
        lowerInto(v, buf.asByteBuffer()!!)
        return buf
    } catch (e: Throwable) {
        RustBuffer.free(buf)
        throw e
    }
}

fun Boolean.lowersIntoSize(): Int {
    return 1
}

fun Boolean.lowerInto(buf: ByteBuffer) {
    buf.put(if (this) 1.toByte() else 0.toByte())
}

fun Int.lower(): Int {
    return this
}

fun Int.lowersIntoSize(): Int {
    return 4
}

fun Int.lowerInto(buf: ByteBuffer) {
    buf.putInt(this)
}

fun Long.lower(): Long {
    return this
}

fun Long.lowersIntoSize(): Int {
    return 8
}

fun Long.lowerInto(buf: ByteBuffer) {
    buf.putLong(this)
}

fun Float.lower(): Float {
    return this
}

fun Float.lowersIntoSize(): Int {
    return 4
}

fun Float.lowerInto(buf: ByteBuffer) {
    buf.putFloat(this)
}

fun Double.lower(): Double {
    return this
}

fun Double.lowersIntoSize(): Int {
    return 8
}

fun Double.lowerInto(buf: ByteBuffer) {
    buf.putDouble(this)
}

fun String.lower(): String {
    return this
}

fun String.lowerInto(buf: ByteBuffer) {
    val byteArr = this.toByteArray()
    buf.putInt(byteArr.size)
    buf.put(byteArr)
}

fun String.Companion.liftFrom(buf: ByteBuffer): String {
    val len = Int.liftFrom(buf)
    val byteArr = ByteArray(len)
    buf.get(byteArr)
    return byteArr.toString(Charsets.UTF_8)
}

fun String.lowersIntoSize(): Int {
    return this.toByteArray().size + 4
}

fun String.Companion.lift(ptr: Pointer): String {
    try {
        return ptr.getString(0, "utf8")
    } finally {
        _UniFFILib.INSTANCE.{{ ci.ffi_string_free().name() }}(ptr)
    }
}

fun<T> lowerSequence(v: List<T>, lowersIntoSize: (T) -> Int, lowerInto: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v -> lowersIntoSizeSequence(v, lowersIntoSize) }, { v, buf -> lowerIntoSequence(v, buf, lowerInto) })
}

fun<T> lowersIntoSizeSequence(v: List<T>, lowersIntoSize: (T) -> Int): Int {
    var len = v.size.lowersIntoSize()
    v.forEach { len += lowersIntoSize(it) }
    return len
}

fun<T> lowerIntoSequence(v: List<T>, buf: ByteBuffer, lowerInto: (T, ByteBuffer) -> Unit) {
    v.size.lowerInto(buf)
    v.forEach { lowerInto(it, buf) }
}

fun<T> lowerOptional(v: T?, lowersIntoSize: (T) -> Int, lowerInto: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v -> lowersIntoSizeOptional(v, lowersIntoSize) }, { v, buf -> lowerIntoOptional(v, buf, lowerInto) })
}

fun<T> lowersIntoSizeOptional(v: T?, lowersIntoSize: (T) -> Int): Int {
    if (v === null) return 1
    return 1 + lowersIntoSize(v)
}

fun<T> lowerIntoOptional(v: T?, buf: ByteBuffer, lowerInto: (T, ByteBuffer) -> Unit) {
    if (v === null) {
        buf.put(0)
    } else {
        buf.put(1)
        lowerInto(v, buf)
    }
}
