{%- let type_name = custom.self_type.type_rs %}

/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ custom.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let builtin_value = {{ custom.builtin.read_fn_rs }}(cursor)?;
    <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::try_lift(builtin_value)
}

/// Write a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ custom.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: {{ type_name }},
) -> uniffi::Result<()> {
    let builtin_value = <{{ type_name }} as uniffi::CustomType<{{ custom.crate_name }}::UniFfiTag>>::lower(value);
    {{ custom.builtin.write_fn_rs }}(cursor, builtin_value)?;
    Ok(())
}

