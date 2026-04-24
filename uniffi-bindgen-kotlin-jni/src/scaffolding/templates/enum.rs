{%- let type_name = en.self_type.type_rs %}

{%- if en.is_flat_error() %}
// Note: no read function, since passing flat errors from Kotlin to Rust is not allowed.

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ en.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    match value {
        {%- for v in en.variants %}
        {{ type_name }}::{{ v.name_rs() }} { .. } => {
            uniffi::FfiBufferCursor::write_u32(cursor, {{ loop.index0 }})?;
            uniffi::FfiBufferCursor::write_string(cursor, value.to_string())?;
        }
        {%- endfor %}
    }
    Ok(())
}
{%- else %}
/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ en.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    Ok(match uniffi::FfiBufferCursor::read_u32(cursor)? {
        {%- for v in en.variants %}
        {{ loop.index0 }} => {
        {%- match v.fields_kind %}
            {%- when FieldsKind::Unit %}
            {{ type_name }}::{{ v.name_rs() }}
            {%- when FieldsKind::Named %}
            {{ type_name }}::{{ v.name_rs() }} {
                {%- for f in v.fields %}
                {{ f.name_rs() }}: {{ f.ty.read_fn_rs }}(cursor)?,
                {%- endfor %}
            }
            {%- when FieldsKind::Unnamed %}
            {{ type_name }}::{{ v.name_rs() }} (
                {%- for f in v.fields %}
                {{ f.ty.read_fn_rs }}(cursor)?,
                {%- endfor %}
            )
            {%- endmatch %}
        }
        {%- endfor %}
        d => uniffi::deps::anyhow::bail!("Invalid {{ type_name }} discriminent: {d}"),
    })
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ en.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    match value {
    {%- for v in en.variants %}
        {%- match v.fields_kind %}
        {%- when FieldsKind::Unit %}
        {{ type_name }}::{{ v.name_rs() }} => {
            uniffi::FfiBufferCursor::write_u32(cursor, {{ loop.index0 }})?;
        }
        {%- when FieldsKind::Named %}
        {{ type_name }}::{{ v.name_rs() }} {
            {%- for f in v.fields %}
            {{ f.name_rs() }},
            {%- endfor %}
        } => {
            uniffi::FfiBufferCursor::write_u32(cursor, {{ loop.index0 }})?;
            {%- for f in v.fields %}
            {{ f.ty.write_fn_rs }}(cursor, {{ f.name_rs() }})?;
            {%- endfor %}
        }
        {%- when FieldsKind::Unnamed %}
        {{ type_name }}::{{ v.name_rs() }} (
            {%- for f in v.fields %}
            v{{ loop.index }},
            {%- endfor %}
        ) => {
            uniffi::FfiBufferCursor::write_u32(cursor, {{ loop.index0 }})?;
            {%- for f in v.fields %}
            {{ f.ty.write_fn_rs }}(cursor, v{{ loop.index }})?;
            {%- endfor %}
        }
        {%- endmatch %}
        {%- endfor %}
    }
    Ok(())
}
{%- endif %}
