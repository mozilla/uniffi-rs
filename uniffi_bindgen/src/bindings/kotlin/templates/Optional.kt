object {{ ffi_converter_name }}: FFIConverterRustBuffer<{{ name }}> {
    override fun write(v: {{ name }}, buf: RustBufferBuilder) {
        if (v == null) {
            buf.putByte(0)
        } else {
            buf.putByte(1)
            {{ inner.write() }}(v, buf)
        }
    }

    override fun read(buf: ByteBuffer): {{ name }} {
        if (buf.get().toInt() == 0) {
            return null
        }
        return {{ inner.read() }}(buf)
    }
}
