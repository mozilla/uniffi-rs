{%- let type_name = rec.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ rec.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    Ok({{ type_name }} {
        {%- for field in rec.fields %}
        {{ field.name_rs() }}: {{ field.ty.read_fn_rs }}(cursor)?,
        {%- endfor %}
    })
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ rec.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    {%- for field in rec.fields %}
    {{ field.ty.write_fn_rs }}(cursor, value.{{ field.name_rs() }})?;
    {%- endfor %}
    Ok(())
}
