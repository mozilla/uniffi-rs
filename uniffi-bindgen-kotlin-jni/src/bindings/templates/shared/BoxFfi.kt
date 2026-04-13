{%- let type_name = box_.self_type.type_kt %}

fun {{ box_.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ box_.inner.read_fn_kt }}(cursor)
}

fun {{ box_.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    {{ box_.inner.write_fn_kt }}(cursor, value)
}

