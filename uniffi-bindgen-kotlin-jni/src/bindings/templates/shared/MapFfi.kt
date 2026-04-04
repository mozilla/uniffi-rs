{%- let type_name = map.self_type.type_kt %}

@JvmName("{{ map.self_type.lower_fn_kt() }}")
fun {{ map.self_type.lower_fn_kt() }}(value: {{ type_name }}): java.nio.ByteBuffer {
    val buf = Scaffolding.ffiBufferAlloc(value.size * {{ map.item_size }}).order(java.nio.ByteOrder.nativeOrder())
    var offset = 0
    value.forEach {
        {{ map.key.write_fn_kt() }}(buf, offset, it.key)
        {{ map.value.write_fn_kt() }}(buf, offset + {{ map.value_offset }}, it.value)
        offset += {{ map.item_size }}
    }
    return buf
}

@JvmName("{{ map.self_type.lift_fn_kt() }}")
fun {{ map.self_type.lift_fn_kt() }}(buf: java.nio.ByteBuffer?): {{ type_name }} {
    val buf = buf!!.order(java.nio.ByteOrder.nativeOrder())
    try {
        val len = buf.capacity() / {{ map.item_size }}
        var offset = 0
        return buildMap(len) {
            for (i in 0..<len) {
                put(
                    {{ map.key.read_fn_kt() }}(buf, offset),
                    {{ map.value.read_fn_kt() }}(buf, offset + {{ map.value_offset }}),
                )
                offset += {{ map.item_size }}
            }
        }
    } finally {
        Scaffolding.ffiBufferFree(buf)
    }
}

fun {{ map.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    val childBuf = {{ map.self_type.lower_fn_kt() }}(value)
    writeBuffer(buf, offset, childBuf)
}

fun {{ map.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val childBuf = readBuffer(buf, offset)
    return {{ map.self_type.lift_fn_kt() }}(childBuf)
}
