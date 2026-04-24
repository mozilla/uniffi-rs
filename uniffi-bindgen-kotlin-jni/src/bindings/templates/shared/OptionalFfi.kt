{%- let type_name = opt.self_type.type_kt %}

fun {{ opt.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    if (readUByte(cursor) == 0.toUByte()) {
        return null
    }
    return {{ opt.inner.read_fn_kt }}(cursor)
}

fun {{ opt.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    if (value == null) {
        writeUByte(cursor, 0.toUByte())
    } else {
        writeUByte(cursor, 1.toUByte())
        {{ opt.inner.write_fn_kt }}(cursor, value)
    }
}
