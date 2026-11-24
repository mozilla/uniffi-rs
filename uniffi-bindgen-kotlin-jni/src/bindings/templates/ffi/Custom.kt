{%- let type_name = custom.self_type.type_kt %}

@JvmName("{{ custom.self_type.lower_fn_kt() }}")
fun {{ custom.self_type.lower_fn_kt() }}(value: {{ type_name }}): {{ custom.builtin.lowered_type_kt() }} {
    {%- if let Some(config) = custom.config %}
    val value = ({{ config.lower("value") }})
    {%- endif %}
    return {{ custom.builtin.lower_fn_kt() }}(value)
}

@JvmName("{{ custom.self_type.lift_fn_kt() }}")
fun {{ custom.self_type.lift_fn_kt() }}(
    {%- for ffi_type in custom.builtin.ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
    {%- endfor %}
): {{ type_name }} {
    val builtinValue = {{ custom.builtin.lift_fn_kt() }}(
        {%- for _ in custom.builtin.ffi_types %}
        v{{ loop.index0 }},
        {%- endfor %}
    )
    {%- match custom.config %}
    {%- when None %}
    return builtinValue
    {%- when Some(config) %}
    return {{ config.lift("builtinValue") }}
    {%- endmatch %}
}

fun {{ custom.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    {%- if let Some(config) = custom.config %}
    val value = ({{ config.lower("value") }})
    {%- endif %}
    {{ custom.builtin.write_fn_kt() }}(buf, offset, value)
}

fun {{ custom.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    val builtinValue = {{ custom.builtin.read_fn_kt() }}(buf, offset)
    {%- match custom.config %}
    {%- when None %}
    return builtinValue
    {%- when Some(config) %}
    return {{ config.lift("builtinValue") }}
    {%- endmatch %}
}
