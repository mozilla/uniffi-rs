// Helper functions for passing values of type {{ name }}
internal fun lower{{ canonical_name }}(v: List<{{ inner.nm() }}>): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_name }}(v, buf)
    }
}

internal fun write{{ canonical_name }}(v: List<{{ inner.nm() }}>, buf: RustBufferBuilder) {
    buf.putInt(v.size)
    v.forEach {
        {{ inner.write("it", "buf") }}
    }
}

internal fun lift{{ canonical_name }}(rbuf: RustBuffer.ByValue): List<{{ inner.nm() }}> {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_name }}(buf)
    }
}

internal fun read{{ canonical_name }}(buf: ByteBuffer): List<{{ inner.nm() }}> {
    val len = buf.getInt()
    return List<{{ inner.nm() }}>(len) {
        {{ inner.read("buf") }}
    }
}
