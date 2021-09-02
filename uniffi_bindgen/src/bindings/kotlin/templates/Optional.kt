// Helper functions for passing values of type {{ name }}

internal fun lower{{ canonical_name }}(v: {{ inner.nm() }}?): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_name }}(v, buf)
    }
}

internal fun write{{ canonical_name }}(v: {{ inner.nm() }}?, buf: RustBufferBuilder) {
    if (v == null) {
        buf.putByte(0)
    } else {
        buf.putByte(1)
        {{ inner.write("v", "buf") }}
    }
}

internal fun lift{{ canonical_name }}(rbuf: RustBuffer.ByValue): {{ inner.nm() }}? {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_name }}(buf)
    }
}

internal fun read{{ canonical_name }}(buf: ByteBuffer): {{ inner.nm() }}? {
    if (buf.get().toInt() == 0) {
        return null
    }
    return {{ inner.read("buf") }}
}
