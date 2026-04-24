{%- let type_name = box_.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ box_.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    Ok(::std::boxed::Box::new({{ box_.inner.read_fn_rs }}(cursor)?))
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
#[allow(clippy::boxed_local)]
pub fn {{ box_.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    {{ box_.inner.write_fn_rs }}(cursor, *value)?;
    Ok(())
}

