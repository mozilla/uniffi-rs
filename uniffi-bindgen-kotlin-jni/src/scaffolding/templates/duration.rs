pub fn uniffi_read_duration(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<::std::time::Duration> {
    Ok(::std::time::Duration::new(cursor.read_u64()?, cursor.read_u32()?))
}

pub fn uniffi_write_duration(
    cursor: &mut uniffi::FfiBufferCursor,
    value: ::std::time::Duration,
) -> uniffi::Result<()> {
    cursor.write_u64(value.as_secs())?;
    cursor.write_u32(value.subsec_nanos())?;
    Ok(())
}
