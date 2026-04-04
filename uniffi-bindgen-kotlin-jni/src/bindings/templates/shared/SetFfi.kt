{%- let type_name = set.self_type.type_kt %}

@JvmName("{{ set.self_type.lower_fn_kt() }}")
fun {{ set.self_type.lower_fn_kt() }}(value: {{ type_name }}): java.nio.ByteBuffer {
    val buf = Scaffolding.ffiBufferAlloc(value.size * {{ set.item_size }}).order(java.nio.ByteOrder.nativeOrder())
    var offset = 0
    value.forEach {
        {{ set.inner.write_fn_kt() }}(buf, offset, it)
        offset += {{ set.item_size }}
    }
    return buf
}

@JvmName("{{ set.self_type.lift_fn_kt() }}")
fun {{ set.self_type.lift_fn_kt() }}(buf: java.nio.ByteBuffer?): {{ type_name }} {
    val buf = buf!!.order(java.nio.ByteOrder.nativeOrder())
    try {
        val len = buf.capacity() / {{ set.item_size }}
        var offset = 0
        return buildSet(len) {
            for (i in 0..<len) {
                add({{ set.inner.read_fn_kt() }}(buf, offset))
                offset += {{ set.item_size }}
            }
        }
    } finally {
        Scaffolding.ffiBufferFree(buf)
    }
}

fun {{ set.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    val childBuf = {{ set.self_type.lower_fn_kt() }}(value)
    writeBuffer(buf, offset, childBuf)
}

fun {{ set.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val childBuf = readBuffer(buf, offset)
    return {{ set.self_type.lift_fn_kt() }}(childBuf)
}
