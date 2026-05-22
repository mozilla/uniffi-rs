{%- let type_name = rec.self_type.type_kt %}

{%- if !rec.self_type.lowers_to_primitive() %}
// Deconstructed version of {{ rec.name }}
class {{ rec.self_type.lowered_type_kt() }}(
    {%- for ffi_type in rec.ffi_types() %}
    val v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
    {%- endfor %}
)
{%- endif %}

@JvmName("{{ rec.self_type.lower_fn_kt() }}")
fun {{ rec.self_type.lower_fn_kt() }}(rec: {{ type_name }}): {{ rec.self_type.lowered_type_kt() }} {
    // Prepare by deconstructing all recursive types
    {%- for field in rec.fields %}
    val uniffiFieldLowered{{ field.index }} = {{ field.ty.lower_fn_kt() }}(rec.{{ field.name_kt() }})
    {%- endfor %}

    {%- if rec.self_type.lowers_to_primitive() %}
    return uniffiFieldLowered0
    {%- else %}
    return {{ rec.self_type.lowered_type_kt() }}(
        {%- for field in rec.fields %}
        {%- for (var, _) in field.ty.ffi_values_kt(format!("uniffiFieldLowered{}", field.index)) %}
        {{ var }},
        {%- endfor %}
        {%- endfor %}
    )
    {%- endif %}
}

@JvmName("{{ rec.self_type.lift_fn_kt() }}")
fun {{ rec.self_type.lift_fn_kt() }}(
    {%- for ffi_type in rec.ffi_types() %}
    v{{ loop.index0 }}: {{ ffi_type.type_kt() }},
    {%- endfor %}
): {{ type_name }} {
    return {{ type_name }}(
        {%- for field in rec.fields %}
        {{ field.ty.lift_fn_kt() }}(
            {%- for ffi_field in field.ffi_fields %}
            v{{ ffi_field.index }},
            {%- endfor %}
        ),
        {%- endfor %}
    )
}
