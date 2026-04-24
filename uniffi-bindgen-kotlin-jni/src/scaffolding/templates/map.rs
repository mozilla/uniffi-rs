{%- let type_name = map.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ map.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let len = cursor.read_u64()? as usize;
    let mut result = ::std::collections::HashMap::with_capacity(len);
    for _ in 0..len {
        result.insert(
            {{ map.key.read_fn_rs}}(cursor)?,
            {{ map.value.read_fn_rs }}(cursor)?
        );
    }
    Ok(result)
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ map.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    cursor.write_u64(value.len() as u64)?;
    for (k, v) in value {
        {{ map.key.write_fn_rs }}(cursor, k)?;
        {{ map.value.write_fn_rs }}(cursor, v)?;
    }
    Ok(())
}
