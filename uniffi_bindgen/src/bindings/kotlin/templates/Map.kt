object {{ ffi_converter_name }}: FFIConverterRustBuffer<{{ name }}> {
    override fun write(v: {{ name }}, buf: RustBufferBuilder) {
        buf.putInt(v.size)
        // The parens on `(k, v)` here ensure we're calling the right method,
        // which is important for compatibility with older android devices.
        // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
        v.forEach { (k, v) ->
            {{ Type::String.write() }}(k, buf)
            {{ inner.write() }}(v, buf)
        }
    }

    override fun read(buf: ByteBuffer): {{ name }} {
        // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
        val items : MutableMap<String, {{ inner.nm() }}> = mutableMapOf()
        val len = buf.getInt()
        repeat(len) {
            val k = {{ Type::String.read() }}(buf)
            val v = {{ inner.read() }}(buf)
            items[k] = v
        }
        return items
    }
}
