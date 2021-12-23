{%- let inner_type_name = inner_type|type_name %}

internal object {{ ffi_converter_name }} {
    internal fun lower(v: List<{{ inner_type_name }}>): RustBuffer.ByValue {
        return lowerIntoRustBuffer(v) { v, buf ->
            write(v, buf)
        }
    }

    internal fun write(v: List<{{ inner_type_name }}>, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        v.forEach {
            {{ inner_type|write_fn }}(it, buf)
        }
    }

    internal fun lift(rbuf: RustBuffer.ByValue): List<{{ inner_type_name }}> {
        return liftFromRustBuffer(rbuf) { buf ->
            read(buf)
        }
    }

    internal fun read(buf: ByteBuffer): List<{{ inner_type_name }}> {
        val len = buf.getInt()
        return List<{{ inner_type_name }}>(len) {
            {{ inner_type|read_fn }}(buf)
        }
    }
}
