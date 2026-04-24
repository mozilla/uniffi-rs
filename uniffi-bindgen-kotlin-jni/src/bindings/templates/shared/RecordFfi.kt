{%- let type_name = rec.self_type.type_kt %}

fun {{ rec.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    return {{ type_name }}(
        {%- for field in rec.fields %}
        {{ field.ty.read_fn_kt }}(cursor),
        {%- endfor %}
    )
}

fun {{ rec.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    {%- for field in rec.fields %}
    {{ field.ty.write_fn_kt }}(cursor, value.{{ field.name_kt() }})
    {%- endfor %}
}
