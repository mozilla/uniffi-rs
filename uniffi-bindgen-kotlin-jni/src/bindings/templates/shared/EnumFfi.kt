{%- let type_name = en.self_type.type_kt %}

{%- match en.kotlin_kind %}
{%- when KotlinEnumKind::EnumClass { .. } %}

fun {{ en.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val uniffiDiscriminent = uniffi.readUInt(cursor).toInt()
    return try {
        {%- if en.use_entries %}
        {{ type_name }}.entries[uniffiDiscriminent]
        {%- else %}
        {{ type_name }}.values()[uniffiDiscriminent]
        {%- endif %}
    } catch (e: IndexOutOfBoundsException) {
        throw uniffi.InternalException("Invalid enum value: ${uniffiDiscriminent}")
    }
}

fun {{ en.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    uniffi.writeUInt(cursor, value.ordinal.toUInt())
}

{%- when KotlinEnumKind::SealedClass %}

fun {{ en.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val uniffiDiscriminent = uniffi.readUInt(cursor)
    return when(uniffiDiscriminent) {
        {%- for v in en.variants %}
        {%- if v.fields.is_empty() && !en.self_type.is_used_as_error %}
        {{ loop.index0 }}u -> {{ type_name }}.{{ v.name_kt }}
        {%- else %}
        {{ loop.index0 }}u -> {{ type_name }}.{{ v.name_kt }}(
            {%- for f in v.fields %}
            {{ f.ty.read_fn_kt }}(cursor),
            {%- endfor %}
        )
        {%- endif %}
        {%- endfor %}
        else -> throw uniffi.InternalException("Invalid enum value: ${uniffiDiscriminent}")
    }
}

fun {{ en.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    when(value) {
        {%- for v in en.variants %}
        is {{ type_name }}.{{ v.name_kt }} -> {
            uniffi.writeUInt(cursor, {{ loop.index0 }}u)
            {%- for f in v.fields %}
            {{ f.ty.write_fn_kt }}(cursor, value.{{ f.name_kt() }})
            {%- endfor %}
            Unit
        }
        {%- endfor %}
    }
}

{%- when KotlinEnumKind::FlatError %}

fun {{ en.self_type.read_fn_kt }}(cursor: uniffi.FfiBufferCursor): {{ type_name }} {
    val uniffiDiscriminent = uniffi.readUInt(cursor)
    return when(uniffiDiscriminent) {
        {%- for v in en.variants %}
        {{ loop.index0 }}u -> {{ type_name }}.{{ v.name_kt }}(uniffi.readString(cursor))
        {%- endfor %}
        else -> throw uniffi.InternalException("Invalid enum value: ${uniffiDiscriminent}")
    }
}

fun {{ en.self_type.write_fn_kt }}(cursor: uniffi.FfiBufferCursor, value: {{ type_name }}) {
    throw uniffi.InternalException("{{ en.self_type.write_fn_kt }}: writing flat errors is not supported")
}

{% endmatch %}
