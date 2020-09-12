// Helpers for reading primitive data types from a bytebuffer.

fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): T {
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

fun Boolean.Companion.lift(v: Byte): Boolean {
    return v.toInt() != 0
}

fun Boolean.Companion.read(buf: ByteBuffer): Boolean {
    return Boolean.lift(buf.get())
}

fun Byte.Companion.lift(v: Byte): Byte {
    return v
}

fun Byte.Companion.read(buf: ByteBuffer): Byte {
    return buf.get()
}

fun Short.Companion.lift(v: Short): Short {
    return v
}

fun Short.Companion.read(buf: ByteBuffer): Short {
    return buf.getShort()
}

fun Int.Companion.lift(v: Int): Int {
    return v
}

fun Int.Companion.read(buf: ByteBuffer): Int {
    return buf.getInt()
}

fun Long.Companion.lift(v: Long): Long {
    return v
}

fun Long.Companion.read(buf: ByteBuffer): Long {
    return buf.getLong()
}

// Unsigned types
fun UByte.Companion.lift(v: Byte): UByte {
    return v.toUByte()
}

fun UByte.Companion.read(buf: ByteBuffer): UByte {
    return UByte.lift(buf.get())
}

fun UShort.Companion.lift(v: Short): UShort {
    return v.toUShort()
}

fun UShort.Companion.read(buf: ByteBuffer): UShort {
    return UShort.lift(buf.getShort())
}

fun UInt.Companion.lift(v: Int): UInt {
    return v.toUInt()
}

fun UInt.Companion.read(buf: ByteBuffer): UInt {
    return UInt.lift(buf.getInt())
}

fun ULong.Companion.lift(v: Long): ULong {
    return v.toULong()
}

fun ULong.Companion.read(buf: ByteBuffer): ULong {
    return ULong.lift(buf.getLong())
}

fun Float.Companion.lift(v: Float): Float {
    return v
}

fun Float.Companion.read(buf: ByteBuffer): Float {
    return buf.getFloat()
}

fun Double.Companion.lift(v: Double): Double {
    return v
}

fun Double.Companion.read(buf: ByteBuffer): Double {
    val v = buf.getDouble()
    return v
}

// I can't figure out how to make a generic implementation of (Any?).read, and IIUC there are some
// restrictions on generics in Kotlin (inherited from the JVM) that make it impossible to write in the
// style I want here. So, we use a standalone helper.

fun<T> liftOptional(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): T? {
    return liftFromRustBuffer(rbuf) { buf -> readOptional(buf, readItem) }
}

fun<T> readOptional(buf: ByteBuffer, readItem: (ByteBuffer) -> T): T? {
    if (! Boolean.read(buf)) {
        return null
    }
    return readItem(buf)
}

fun<T> liftSequence(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): List<T> {
    return liftFromRustBuffer(rbuf) { buf -> readSequence(buf, readItem) }
}

fun<T> readSequence(buf: ByteBuffer, readItem: (ByteBuffer) -> T): List<T> {
    val len = Int.read(buf)
    return List<T>(len) {
        readItem(buf)
    }
}

fun<V> liftMap(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> Pair<String, V>): Map<String, V> {
    return liftFromRustBuffer(rbuf) { buf -> readMap(buf, readItem) }
}

fun<V> readMap(buf: ByteBuffer, readItem: (ByteBuffer) -> Pair<String, V>): Map<String, V> {
    val len = Int.read(buf)
    @OptIn(ExperimentalStdlibApi::class)
    return buildMap<String, V>(len) {
        repeat(len) {
            val (k, v) = readItem(buf)
            put(k, v)
        }
    }
}

// Helpers for lowering primitive data types into a RustBuffer.

fun<T> lowerIntoRustBuffer(v: T, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
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

fun Boolean.lower(): Byte {
    return if (this) 1.toByte() else 0.toByte()
}

fun Boolean.write(buf: RustBufferBuilder) {
    buf.putByte(this.lower())
}

fun Byte.lower(): Byte {
    return this
}

fun Byte.write(buf: RustBufferBuilder) {
    buf.putByte(this)
}

fun Short.lower(): Short {
    return this
}

fun Short.write(buf: RustBufferBuilder) {
    buf.putShort(this)
}

fun Int.lower(): Int {
    return this
}

fun Int.write(buf: RustBufferBuilder) {
    buf.putInt(this)
}

fun Long.lower(): Long {
    return this
}

fun Long.write(buf: RustBufferBuilder) {
    buf.putLong(this)
}

// Experimental unsigned types
fun UByte.lower(): Byte {
    return this.toByte()
}

fun UByte.write(buf: RustBufferBuilder) {
    buf.putByte(this.toByte())
}

fun UShort.lower(): Short {
    return this.toShort()
}

fun UShort.write(buf: RustBufferBuilder) {
    buf.putShort(this.toShort())
}

fun UInt.lower(): Int {
    return this.toInt()
}

fun UInt.write(buf: RustBufferBuilder) {
    buf.putInt(this.toInt())
}

fun ULong.lower(): Long {
    return this.toLong()
}

fun ULong.write(buf: RustBufferBuilder) {
    buf.putLong(this.toLong())
}

fun Float.lower(): Float {
    return this
}

fun Float.write(buf: RustBufferBuilder) {
    buf.putFloat(this)
}

fun Double.lower(): Double {
    return this
}

fun Double.write(buf: RustBufferBuilder) {
    buf.putDouble(this)
}

fun String.lower(): Pointer {
    val rustErr = RustError.ByReference()
    val rustStr = _UniFFILib.INSTANCE.{{ ci.ffi_string_alloc_from().name() }}(this, rustErr)
    if (rustErr.code != 0) {
         throw RuntimeException("caught a panic while passing a string across the ffi")
    }
    return rustStr
}

fun String.write(buf: RustBufferBuilder) {
    val byteArr = this.toByteArray(Charsets.UTF_8)
    buf.putInt(byteArr.size)
    buf.put(byteArr)
}

fun String.Companion.read(buf: ByteBuffer): String {
    val len = Int.read(buf)
    val byteArr = ByteArray(len)
    buf.get(byteArr)
    return byteArr.toString(Charsets.UTF_8)
}

fun String.Companion.lift(ptr: Pointer): String {
    try {
        return ptr.getString(0, "utf8")
    } finally {
        rustCall(InternalError.ByReference()) { err ->
            _UniFFILib.INSTANCE.{{ ci.ffi_string_free().name() }}(ptr, err)
        }
    }
}

fun<T> lowerSequence(v: List<T>, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v, buf -> writeSequence(v, buf, writeItem) })
}

fun<T> writeSequence(v: List<T>, buf: RustBufferBuilder, writeItem: (T, RustBufferBuilder) -> Unit) {
    v.size.write(buf)
    v.forEach { writeItem(it, buf) }
}

fun<V> lowerMap(m: Map<String, V>, writeEntry: (String, V, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(m, { m, buf -> writeMap(m, buf, writeEntry) })
}

fun<V> writeMap(v: Map<String, V>, buf: RustBufferBuilder, writeEntry: (String, V, RustBufferBuilder) -> Unit) {
    v.size.write(buf)
    v.forEach { k, v -> writeEntry(k, v, buf) }
}

fun<T> lowerOptional(v: T?, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v, buf -> writeOptional(v, buf, writeItem) })
}

fun<T> writeOptional(v: T?, buf: RustBufferBuilder, writeItem: (T, RustBufferBuilder) -> Unit) {
    if (v === null) {
        buf.putByte(0)
    } else {
        buf.putByte(1)
        writeItem(v, buf)
    }
}
