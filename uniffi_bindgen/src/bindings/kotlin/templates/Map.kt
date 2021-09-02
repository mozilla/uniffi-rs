// Helper functions for passing values of type {{ name }}
internal fun lower{{ canonical_name }}(m: Map<String, {{ inner.nm() }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(m) { m, buf ->
        write{{ canonical_name }}(m, buf)
    }
}

internal fun write{{ canonical_name }}(v: Map<String, {{ inner.nm() }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    // The parens on `(k, v)` here ensure we're calling the right method,
    // which is important for compatibility with older android devices.
    // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
    v.forEach { (k, v) ->
        {{ Type::String.write("k", "buf") }}
        {{ inner.write("v", "buf") }}
    }
}

internal fun lift{{ canonical_name }}(rbuf: RustBuffer.ByValue): Map<String, {{ inner.nm() }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_name }}(buf)
    }
}

internal fun read{{ canonical_name }}(buf: ByteBuffer): Map<String, {{ inner.nm() }}> {
    // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
    val items : MutableMap<String, {{ inner.nm() }}> = mutableMapOf()
    val len = buf.getInt()
    repeat(len) {
        val k = {{ Type::String.read("buf") }}
        val v = {{ inner.read("buf") }}
        items[k] = v
    }
    return items
}
