// Helpers for reading primitive data types from a bytebuffer.

internal fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): T {
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

internal fun<T> lowerIntoRustBuffer(v: T, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
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

// For every type used in the interface, we provide helper methods for conveniently
// lifting and lowering that type from C-compatible data, and for reading and writing
// values of that type in a buffer.

{% for typ in ci.iter_types() %}
{% let canonical_type_name = typ.canonical_name()|class_name_kt %}
{%- match typ -%}

{% when Type::Boolean -%}

internal fun Boolean.Companion.lift(v: Byte): Boolean {
    return v.toInt() != 0
}

internal fun Boolean.Companion.read(buf: ByteBuffer): Boolean {
    return Boolean.lift(buf.get())
}

internal fun Boolean.lower(): Byte {
    return if (this) 1.toByte() else 0.toByte()
}

internal fun Boolean.write(buf: RustBufferBuilder) {
    buf.putByte(this.lower())
}

{% when Type::Int8 -%}

internal fun Byte.Companion.lift(v: Byte): Byte {
    return v
}

internal fun Byte.Companion.read(buf: ByteBuffer): Byte {
    return buf.get()
}

internal fun Byte.lower(): Byte {
    return this
}

internal fun Byte.write(buf: RustBufferBuilder) {
    buf.putByte(this)
}

{% when Type::Int16 -%}

internal fun Short.Companion.lift(v: Short): Short {
    return v
}

internal fun Short.Companion.read(buf: ByteBuffer): Short {
    return buf.getShort()
}

internal fun Short.lower(): Short {
    return this
}

internal fun Short.write(buf: RustBufferBuilder) {
    buf.putShort(this)
}

{% when Type::Int32 -%}

internal fun Int.Companion.lift(v: Int): Int {
    return v
}

internal fun Int.Companion.read(buf: ByteBuffer): Int {
    return buf.getInt()
}

internal fun Int.lower(): Int {
    return this
}

internal fun Int.write(buf: RustBufferBuilder) {
    buf.putInt(this)
}

{% when Type::Int64 -%}

internal fun Long.Companion.lift(v: Long): Long {
    return v
}

internal fun Long.Companion.read(buf: ByteBuffer): Long {
    return buf.getLong()
}

internal fun Long.lower(): Long {
    return this
}

internal fun Long.write(buf: RustBufferBuilder) {
    buf.putLong(this)
}

{% when Type::UInt8 -%}

@ExperimentalUnsignedTypes
internal fun UByte.Companion.lift(v: Byte): UByte {
    return v.toUByte()
}

@ExperimentalUnsignedTypes
internal fun UByte.Companion.read(buf: ByteBuffer): UByte {
    return UByte.lift(buf.get())
}

@ExperimentalUnsignedTypes
internal fun UByte.lower(): Byte {
    return this.toByte()
}

@ExperimentalUnsignedTypes
internal fun UByte.write(buf: RustBufferBuilder) {
    buf.putByte(this.toByte())
}

{% when Type::UInt16 -%}

@ExperimentalUnsignedTypes
internal fun UShort.Companion.lift(v: Short): UShort {
    return v.toUShort()
}

@ExperimentalUnsignedTypes
internal fun UShort.Companion.read(buf: ByteBuffer): UShort {
    return UShort.lift(buf.getShort())
}

@ExperimentalUnsignedTypes
internal fun UShort.lower(): Short {
    return this.toShort()
}

@ExperimentalUnsignedTypes
internal fun UShort.write(buf: RustBufferBuilder) {
    buf.putShort(this.toShort())
}

{% when Type::UInt32 -%}

@ExperimentalUnsignedTypes
internal fun UInt.Companion.lift(v: Int): UInt {
    return v.toUInt()
}

@ExperimentalUnsignedTypes
internal fun UInt.Companion.read(buf: ByteBuffer): UInt {
    return UInt.lift(buf.getInt())
}

@ExperimentalUnsignedTypes
internal fun UInt.lower(): Int {
    return this.toInt()
}

@ExperimentalUnsignedTypes
internal fun UInt.write(buf: RustBufferBuilder) {
    buf.putInt(this.toInt())
}

{% when Type::UInt64 -%}

@ExperimentalUnsignedTypes
internal fun ULong.Companion.lift(v: Long): ULong {
    return v.toULong()
}

@ExperimentalUnsignedTypes
internal fun ULong.Companion.read(buf: ByteBuffer): ULong {
    return ULong.lift(buf.getLong())
}

@ExperimentalUnsignedTypes
internal fun ULong.lower(): Long {
    return this.toLong()
}

@ExperimentalUnsignedTypes
internal fun ULong.write(buf: RustBufferBuilder) {
    buf.putLong(this.toLong())
}

{% when Type::Float32 -%}

internal fun Float.Companion.lift(v: Float): Float {
    return v
}

internal fun Float.Companion.read(buf: ByteBuffer): Float {
    return buf.getFloat()
}

internal fun Float.lower(): Float {
    return this
}

internal fun Float.write(buf: RustBufferBuilder) {
    buf.putFloat(this)
}

{% when Type::Float64 -%}

internal fun Double.Companion.lift(v: Double): Double {
    return v
}

internal fun Double.Companion.read(buf: ByteBuffer): Double {
    val v = buf.getDouble()
    return v
}

internal fun Double.lower(): Double {
    return this
}

internal fun Double.write(buf: RustBufferBuilder) {
    buf.putDouble(this)
}

{% when Type::String -%}

internal fun String.Companion.lift(rbuf: RustBuffer.ByValue): String {
    try {
        val byteArr = ByteArray(rbuf.len)
        rbuf.asByteBuffer()!!.get(byteArr)
        return byteArr.toString(Charsets.UTF_8)
    } finally {
        RustBuffer.free(rbuf)
    }
}

internal fun String.Companion.read(buf: ByteBuffer): String {
    val len = buf.getInt()
    val byteArr = ByteArray(len)
    buf.get(byteArr)
    return byteArr.toString(Charsets.UTF_8)
}

internal fun String.lower(): RustBuffer.ByValue {
    val byteArr = this.toByteArray(Charsets.UTF_8)
    // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
    // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
    val rbuf = RustBuffer.alloc(byteArr.size)
    rbuf.asByteBuffer()!!.put(byteArr)
    return rbuf
}

internal fun String.write(buf: RustBufferBuilder) {
    val byteArr = this.toByteArray(Charsets.UTF_8)
    buf.putInt(byteArr.size)
    buf.put(byteArr)
}

{% when Type::Timestamp -%}
{% let type_name = typ|type_kt %}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): {{ type_name }} {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): {{ type_name }} {
    val seconds = buf.getLong()
    // Type mismatch (should be u32) but we check for overflow/underflow below
    val nanoseconds = buf.getInt().toLong()
    if (nanoseconds < 0) {
        throw java.time.DateTimeException("Instant nanoseconds exceed minimum or maximum supported by uniffi")
    }
    if (seconds >= 0) {
        return {{ type_name }}.EPOCH.plus(java.time.Duration.ofSeconds(seconds, nanoseconds))
    } else {
        return {{ type_name }}.EPOCH.minus(java.time.Duration.ofSeconds(-seconds, nanoseconds))
    }
}

internal fun lower{{ canonical_type_name }}(v: {{ type_name }}): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: {{ type_name }}, buf: RustBufferBuilder) {
    var epoch_offset = java.time.Duration.between({{ type_name }}.EPOCH, v)

    var sign = 1
    if (epoch_offset.isNegative()) {
        sign = -1
        epoch_offset = epoch_offset.negated()
    }

    if (epoch_offset.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Instant.html
        throw IllegalArgumentException("Invalid timestamp, nano value must be non-negative")
    }

    buf.putLong(sign * epoch_offset.seconds)
    // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
    buf.putInt(epoch_offset.nano)
}

{% when Type::Duration -%}
{% let type_name = typ|type_kt %}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): {{ type_name }} {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): {{ type_name }} {
    // Type mismatch (should be u64) but we check for overflow/underflow below
    val seconds = buf.getLong()
    // Type mismatch (should be u32) but we check for overflow/underflow below
    val nanoseconds = buf.getInt().toLong()
    if (seconds < 0) {
        throw java.time.DateTimeException("Duration exceeds minimum or maximum value supported by uniffi")
    }
    if (nanoseconds < 0) {
        throw java.time.DateTimeException("Duration nanoseconds exceed minimum or maximum supported by uniffi")
    }
    return {{ type_name }}.ofSeconds(seconds, nanoseconds)
}

internal fun lower{{ canonical_type_name }}(v: {{ type_name }}): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: {{ type_name }}, buf: RustBufferBuilder) {
    if (v.seconds < 0) {
        // Rust does not support negative Durations
        throw IllegalArgumentException("Invalid duration, must be non-negative")
    }

    if (v.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Duration.html
        throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
    }

    // Type mismatch (should be u64) but since Rust doesn't support negative durations we should be OK
    buf.putLong(v.seconds)
    // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
    buf.putInt(v.nano)
}

{% when Type::Optional with (inner_type) -%}
{% let inner_type_name = inner_type|type_kt %}

// Helper functions for pasing values of type {{ typ|type_kt }}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): {{ inner_type_name }}? {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): {{ inner_type_name }}? {
    if (buf.get().toInt() == 0) {
        return null
    }
    return {{ "buf"|read_kt(inner_type) }}
}

internal fun lower{{ canonical_type_name }}(v: {{ inner_type_name }}?): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: {{ inner_type_name }}?, buf: RustBufferBuilder) {
    if (v == null) {
        buf.putByte(0)
    } else {
        buf.putByte(1)
        {{ "v"|write_kt("buf", inner_type) }}
    }
}

{% when Type::Sequence with (inner_type) -%}
{% let inner_type_name = inner_type|type_kt %}

// Helper functions for pasing values of type {{ typ|type_kt }}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): List<{{ inner_type_name }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): List<{{ inner_type_name }}> {
    val len = buf.getInt()
    return List<{{ inner_type|type_kt }}>(len) {
        {{ "buf"|read_kt(inner_type) }}
    }
}

internal fun lower{{ canonical_type_name }}(v: List<{{ inner_type_name }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: List<{{ inner_type_name }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    v.forEach {
        {{ "it"|write_kt("buf", inner_type) }}
    }
}

{% when Type::Map with (inner_type) -%}
{% let inner_type_name = inner_type|type_kt %}

// Helper functions for pasing values of type {{ typ|type_kt }}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): Map<String, {{ inner_type_name }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
    // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
    val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
    val len = buf.getInt()
    repeat(len) {
        val k = String.read(buf)
        val v = {{ "buf"|read_kt(inner_type) }}
        items[k] = v
    }
    return items
}

internal fun lower{{ canonical_type_name }}(m: Map<String, {{ inner_type_name }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(m) { m, buf ->
        write{{ canonical_type_name }}(m, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: Map<String, {{ inner_type_name }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    // The parens on `(k, v)` here ensure we're calling the right method,
    // which is important for compatibility with older android devices.
    // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
    v.forEach { (k, v) ->
        k.write(buf)
        {{ "v"|write_kt("buf", inner_type) }}
    }
}

{% when Type::Enum with (enum_name) -%}
{# Helpers for Enum types are defined inline with the Enum class #}

{% when Type::Record with (record_name) -%}
{# Helpers for Record types are defined inline with the Record class #}

{% when Type::Object with (object_name) -%}
{# Object types cannot be lifted, lowered or serialized (yet) #}

{% when Type::CallbackInterface with (interface_name) -%}
{# Helpers for Callback Interface types are defined inline with the CallbackInterface class #}

{% else %}
{# This type cannot be lifted, lowered or serialized (yet) #}

{% endmatch %}
{% endfor %}
