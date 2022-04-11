{%- let inner_type_name = inner_type|type_name %}

public object {{ ffi_converter_name }}: FfiConverterRustBuffer<Map<String, {{ inner_type_name }}>> {
    override fun read(buf: ByteBuffer): Map<String, {{ inner_type_name }}> {
        // TODO: Once Kotlin's `buildMap` API is stabilized we should use it here.
        val items : MutableMap<String, {{ inner_type_name }}> = mutableMapOf()
        val len = buf.getInt()
        repeat(len) {
            val k = {{ TypeIdentifier::String.borrow()|read_fn }}(buf)
            val v = {{ inner_type|read_fn }}(buf)
            items[k] = v
        }
        return items
    }

    override fun allocationSize(value: Map<String, {{ inner_type_name }}>): Int {
        val spaceForMapSize = 4
        val spaceForChildren = value.map { (k, v) ->
            {{ TypeIdentifier::String.borrow()|allocation_size_fn }}(k) +
            {{ inner_type|allocation_size_fn }}(v)
        }.sum()
        return spaceForMapSize + spaceForChildren
    }

    override fun write(value: Map<String, {{ inner_type_name }}>, buf: ByteBuffer) {
        buf.putInt(value.size)
        // The parens on `(k, v)` here ensure we're calling the right method,
        // which is important for compatibility with older android devices.
        // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
        value.forEach { (k, v) ->
            {{ TypeIdentifier::String.borrow()|write_fn }}(k, buf)
            {{ inner_type|write_fn }}(v, buf)
        }
    }
}
