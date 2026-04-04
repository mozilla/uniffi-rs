{%- let type_name = seq.self_type.type_kt %}

@JvmName("{{ seq.self_type.lower_fn_kt() }}")
fun {{ seq.self_type.lower_fn_kt() }}(value: {{ type_name }}): java.nio.ByteBuffer {
    val buf = Scaffolding.ffiBufferAlloc(value.size * {{ seq.item_size }}).order(java.nio.ByteOrder.nativeOrder())
    var offset = 0
    value.forEach {
        {{ seq.inner.write_fn_kt() }}(buf, offset, it)
        offset += {{ seq.item_size }}
    }
    return buf
}

@JvmName("{{ seq.self_type.lift_fn_kt() }}")
fun {{ seq.self_type.lift_fn_kt() }}(buf: java.nio.ByteBuffer?): {{ type_name }} {
    val buf = buf!!.order(java.nio.ByteOrder.nativeOrder())
    try {
        val len = buf.capacity() / {{ seq.item_size }}
        var offset = 0
        return buildList(len) {
            for (i in 0..<len) {
                add({{ seq.inner.read_fn_kt() }}(buf, offset))
                offset += {{ seq.item_size }}
            }
        }
    } finally {
        Scaffolding.ffiBufferFree(buf)
    }
}

fun {{ seq.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    val childBuf = {{ seq.self_type.lower_fn_kt() }}(value)
    writeBuffer(buf, offset, childBuf)
}

fun {{ seq.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val childBuf = readBuffer(buf, offset)
    return {{ seq.self_type.lift_fn_kt() }}(childBuf)
}
