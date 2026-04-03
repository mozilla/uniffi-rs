{%- let type_name = cls.self_type.type_kt %}

{# Simple case: the interface can only be implemented in Rust #}

fun {{ cls.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ type_name }}(uniffi.WithHandle, uniffi.readLong(cursor))
}

fun {{ cls.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    value.uniffiAddRef()
    uniffi.writeLong(cursor, value.uniffiHandle)
}
