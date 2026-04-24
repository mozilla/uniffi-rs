{%- let type_name = seq.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ seq.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let len = cursor.read_u64()? as usize;
    let mut result = ::std::vec::Vec::with_capacity(len);
    for _ in 0..len {
        result.push({{ seq.inner.read_fn_rs }}(cursor)?);
    }
    Ok(result)
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
pub fn {{ seq.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    cursor.write_u64(value.len() as u64)?;
    for v in value {
        {{ seq.inner.write_fn_rs }}(cursor, v)?;
    }
    Ok(())
}
