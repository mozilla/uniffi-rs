object {{ ffi_converter_name }}: FFIConverterRustBuffer<{{ name }}> {
    override fun write(v: {{ name }}, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        v.forEach {
            {{ inner.write() }}(it, buf)
        }
    }

    override fun read(buf: ByteBuffer): {{ name }} {
        val len = buf.getInt()
        return {{ name }}(len) {
            {{ inner.read() }}(buf)
        }
    }
}
