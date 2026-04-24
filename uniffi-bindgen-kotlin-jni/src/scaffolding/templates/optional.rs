{%- let type_name = opt.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ opt.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    Ok(match cursor.read_u8()? {
        0 => None,
        1 => Some({{ opt.inner.read_fn_rs }}(cursor)?),
        n => uniffi::deps::anyhow::bail!("{{ opt.self_type.read_fn_rs }}: invalid discriminent: {n}"),
    })
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ opt.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    match value {
        None => cursor.write_u8(0)?,
        Some(inner_value) => {
            cursor.write_u8(1)?;
            {{ opt.inner.write_fn_rs }}(cursor, inner_value)?
        }
    }
    Ok(())
}
