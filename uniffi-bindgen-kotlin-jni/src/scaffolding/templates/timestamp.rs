
pub fn uniffi_read_timestamp(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<::std::time::SystemTime> {
    let seconds = cursor.read_i64()?;
    let nanos = cursor.read_u32()?;
    let epoch_offset = ::std::time::Duration::new(seconds.wrapping_abs() as u64, nanos);

    if seconds >= 0 {
        Ok(::std::time::SystemTime::UNIX_EPOCH + epoch_offset)
    } else {
        Ok(::std::time::SystemTime::UNIX_EPOCH - epoch_offset)
    }
}

pub fn uniffi_write_timestamp(
    cursor: &mut uniffi::FfiBufferCursor,
    value: ::std::time::SystemTime,
) -> uniffi::Result<()> {
    let mut sign = 1;
    let epoch_offset = value
        .duration_since(::std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|error| {
            sign = -1;
            error.duration()
        });
    // This panic should never happen as SystemTime typically stores seconds as i64
    let seconds = sign
        * i64::try_from(epoch_offset.as_secs())
            .expect("SystemTime overflow, seconds greater than i64::MAX");

    cursor.write_i64(seconds)?;
    cursor.write_u32(epoch_offset.subsec_nanos())?;
    Ok(())
}

