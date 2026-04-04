{%- let type_name = opt.self_type.type_kt %}

{%- if !opt.self_type.lowers_to_primitive() %}
{%- let all_ffi_types = opt.self_type.ffi_types %}
{%- let inner_ffi_types = opt.inner.ffi_types %}

// Deconstructed version of {{ type_name }}
class {{ opt.self_type.lowered_type_kt() }}(
    {%- for ffi_type in all_ffi_types %}
    val v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
    {%- endfor %}
)

@JvmName("{{ opt.self_type.lower_fn_kt() }}")
fun {{ opt.self_type.lower_fn_kt() }}(value: {{ type_name }}): {{ opt.self_type.lowered_type_kt() }} {
    if (value == null) {
        return {{ opt.self_type.lowered_type_kt() }}(
            false,
            {%- for ffi_type in inner_ffi_types %}
            {{ ffi_type.default_kt() }},
            {%- endfor %}
        )
    } else {
        val uniffiFieldLowered = {{ opt.inner.lower_fn_kt() }}(value)
        return {{ opt.self_type.lowered_type_kt() }}(
            true,
            {%- for (var, _) in opt.inner.ffi_values_kt("uniffiFieldLowered") %}
            {{ var }},
            {%- endfor %}
        )
    }
}

@JvmName("{{ opt.self_type.lift_fn_kt() }}")
fun {{ opt.self_type.lift_fn_kt() }}(
    {%- for ffi_type in all_ffi_types %}
    v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
    {%- endfor %}
): {{ type_name }} {
    if (v0) {
        return {{ opt.inner.lift_fn_kt() }}(
            {%- for _ in inner_ffi_types %}
            v{{ loop.index0 + 1 }},
            {%- endfor %}
        )
    } else {
        return null
    }
}
{%- endif %}

fun {{ opt.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    if (value == null) {
        writeBoolean(buf, offset, false)
    } else {
        writeBoolean(buf, offset, true)
        {{ opt.inner.write_fn_kt() }}(buf, offset + {{ opt.some_offset }}, value)
    }
}

fun {{ opt.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    return when (readBoolean(buf, offset)) {
        false -> null
        true -> {{ opt.inner.read_fn_kt() }}(buf, offset + {{ opt.some_offset }})
    }
}
