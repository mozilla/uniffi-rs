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

fun Short.Companion.liftFrom(buf: ByteBuffer): Short {
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

// Helpers for lowering primitive data types into a bytebuffer.
// Since we need to allocate buffers from rust, the lowering process needs to be
// able to calculate ahead-of-time what the required size of the buffer will be.

fun<T> lowerIntoRustBuffer(v: T, calculateWriteSize: (T) -> Int, writeItem: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    val buf = RustBuffer.alloc(calculateWriteSize(v))
    try {
        writeItem(v, buf.asByteBuffer()!!)
        return buf
    } catch (e: Throwable) {
        RustBuffer.free(buf)
        throw e
    }
}

fun Boolean.lower(): Byte {
    return if (this) 1.toByte() else 0.toByte()
}

fun Boolean.calculateWriteSize(): Int {
    return 1
}

fun Boolean.write(buf: ByteBuffer) {
    buf.put(this.lower())
}

fun Byte.lower(): Byte {
    return this
}

fun Byte.calculateWriteSize(): Byte {
    return 1
}

fun Byte.write(buf: ByteBuffer) {
    buf.put(this)
}

fun Short.lower(): Short {
    return this
}

fun Short.lowersIntoSize(): Int {
    return 2
}

fun Short.lowerInto(buf: ByteBuffer) {
    buf.putShort(this)
}

fun Int.lower(): Int {
    return this
}

fun Int.calculateWriteSize(): Int {
    return 4
}

fun Int.write(buf: ByteBuffer) {
    buf.putInt(this)
}

fun Long.lower(): Long {
    return this
}

fun Long.calculateWriteSize(): Int {
    return 8
}

fun Long.write(buf: ByteBuffer) {
    buf.putLong(this)
}

fun Float.lower(): Float {
    return this
}

fun Float.calculateWriteSize(): Int {
    return 4
}

fun Float.write(buf: ByteBuffer) {
    buf.putFloat(this)
}

fun Double.lower(): Double {
    return this
}

fun Double.calculateWriteSize(): Int {
    return 8
}

fun Double.write(buf: ByteBuffer) {
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

fun String.write(buf: ByteBuffer) {
    val byteArr = this.toByteArray()
    buf.putInt(byteArr.size)
    buf.put(byteArr)
}

fun String.Companion.read(buf: ByteBuffer): String {
    val len = Int.read(buf)
    val byteArr = ByteArray(len)
    buf.get(byteArr)
    return byteArr.toString(Charsets.UTF_8)
}

fun String.calculateWriteSize(): Int {
    return 4 + this.toByteArray().size
}

fun String.Companion.lift(ptr: Pointer): String {
    try {
        return ptr.getString(0, "utf8")
    } finally {
        _UniFFILib.INSTANCE.{{ ci.ffi_string_free().name() }}(ptr)
    }
}

fun<T> lowerSequence(v: List<T>, calculateWriteSize: (T) -> Int, writeItem: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v -> calculateWriteSizeSequence(v, calculateWriteSize) }, { v, buf -> writeSequence(v, buf, writeItem) })
}

fun<T> calculateWriteSizeSequence(v: List<T>, calculateWriteSize: (T) -> Int): Int {
    var len = v.size.calculateWriteSize()
    v.forEach { len += calculateWriteSize(it) }
    return len
}

fun<T> writeSequence(v: List<T>, buf: ByteBuffer, writeItem: (T, ByteBuffer) -> Unit) {
    v.size.write(buf)
    v.forEach { writeItem(it, buf) }
}

fun<V> lowerMap(m: Map<String, V>, calculateWriteSize: (String, V) -> Int, writeEntry: (String, V, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(m, { m -> calculateWriteSizeMap(m, calculateWriteSize) }, { m, buf -> writeMap(m, buf, writeEntry) })
}

fun<V> calculateWriteSizeMap(v: Map<String, V>, calculateWriteSize: (String, V) -> Int): Int {
    var len = v.size.calculateWriteSize()
    v.forEach { k, v -> len += calculateWriteSize(k, v) }
    return len
}

fun<V> writeMap(v: Map<String, V>, buf: ByteBuffer, writeEntry: (String, V, ByteBuffer) -> Unit) {
    v.size.write(buf)
    v.forEach { k, v -> writeEntry(k, v, buf) }
}

fun<T> lowerOptional(v: T?, calculateWriteSize: (T) -> Int, writeItem: (T, ByteBuffer) -> Unit): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v, { v -> calculateWriteSizeOptional(v, calculateWriteSize) }, { v, buf -> writeOptional(v, buf, writeItem) })
}

fun<T> calculateWriteSizeOptional(v: T?, calculateWriteSize: (T) -> Int): Int {
    if (v === null) return 1
    return 1 + calculateWriteSize(v)
}

fun<T> writeOptional(v: T?, buf: ByteBuffer, writeItem: (T, ByteBuffer) -> Unit) {
    if (v === null) {
        buf.put(0)
    } else {
        buf.put(1)
        writeItem(v, buf)
    }
}
