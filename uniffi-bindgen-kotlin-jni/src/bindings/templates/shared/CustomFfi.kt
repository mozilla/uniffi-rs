{%- let type_name = custom.self_type.type_kt %}

{%- match custom.config %}
{%- when None %}

fun {{ custom.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ custom.builtin.read_fn_kt }}(cursor)
}

fun {{ custom.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    {{ custom.builtin.write_fn_kt }}(cursor, value)
}

{%- when Some(config) %}

fun {{ custom.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val builtinValue = {{ custom.builtin.read_fn_kt }}(cursor)
    return {{ config.lift("builtinValue") }}
}

fun {{ custom.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    val builtinValue = {{ config.lower("value") }}
    {{ custom.builtin.write_fn_kt }}(cursor, builtinValue)
}
{%- endmatch %}
