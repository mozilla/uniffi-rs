{%- let type_name = map.self_type.type_kt %}

fun {{ map.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val len = readULong(cursor).toInt()
    return buildMap<{{ map.key.type_kt }}, {{ map.value.type_kt }}>(len) {
        repeat(len) {
            val k = {{ map.key.read_fn_kt }}(cursor)
            val v = {{ map.value.read_fn_kt }}(cursor)
            this[k] = v
        }
    }
}

fun {{ map.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    writeULong(cursor, value.size.toULong())
    // The parens on `(k, v)` here ensure we're calling the right method,
    // which is important for compatibility with older android devices.
    // Ref https://blog.danlew.net/2017/03/16/kotlin-puzzler-whose-line-is-it-anyways/
    value.forEach { (k, v) ->
        {{ map.key.write_fn_kt }}(cursor, k)
        {{ map.value.write_fn_kt }}(cursor, v)
    }
}

