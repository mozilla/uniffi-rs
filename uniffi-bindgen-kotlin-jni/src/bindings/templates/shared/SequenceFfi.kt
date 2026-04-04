{%- let type_name = seq.self_type.type_kt %}

fun {{ seq.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val len = readULong(cursor).toInt()
    return List<{{ seq.inner.type_kt }}>(len) {
        {{ seq.inner.read_fn_kt }}(cursor)
    }
}

fun {{ seq.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    writeULong(cursor, value.size.toULong())
    value.iterator().forEach {
        {{ seq.inner.write_fn_kt }}(cursor, it)
    }
}
