{%- let type_name = ext.self_type.type_kt %}

{# Forward lift/lower functions to the original definition #}
@JvmName("{{ ext.self_type.lower_fn_kt() }}")
inline fun {{ ext.self_type.lower_fn_kt() }}(value: {{ type_name }}): {{ ext.self_type.lowered_type_kt() }} {
    return {{ ext.package_name }}.{{ ext.self_type.lower_fn_kt() }}(value)
}

@JvmName("{{ ext.self_type.lift_fn_kt() }}")
inline fun {{ ext.self_type.lift_fn_kt() }}(
    {%- for ffi_type in ext.self_type.ffi_types %}
    v{{ loop.index }}: {{ ffi_type.type_kt() }}{% if !loop.last %},{% endif %}
    {%- endfor %}
): {{ type_name }} {
    return {{ ext.package_name }}.{{ ext.self_type.lift_fn_kt() }}(
        {%- for ffi_type in ext.self_type.ffi_types %}
        v{{ loop.index }}{% if !loop.last %},{% endif %}
        {%- endfor %}
    )
}

inline fun {{ ext.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    {{ ext.package_name }}.{{ ext.self_type.write_fn_kt() }}(buf, offset, value)
}

inline fun {{ ext.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    return {{ ext.package_name }}.{{ ext.self_type.read_fn_kt() }}(buf, offset)
}

{# Other lift/lower functions #}
{%- match ext.self_type.ty %}
{%- when Type::Interface { imp, .. } %}
{%- if !imp.has_callback_interface() %}
{%- let lower_object_ref = "lowerObjectRef{}"|format(ext.self_type.id) %}
@JvmName("{{ lower_object_ref }}")
inline fun {{ lower_object_ref }}(value: {{ type_name }}): kotlin.Long {
    return {{ ext.package_name }}.{{ lower_object_ref }}(value)
}
{%- endif %}
{%- else %}
{%- endmatch %}
