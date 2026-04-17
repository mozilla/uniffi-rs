pub fn uniffi_read_bytes(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<Vec<u8>> {
    let len = cursor.read_u64()? as usize;
    let mut result = ::std::vec::Vec::with_capacity(len);
    for _ in 0..len {
        result.push(cursor.read_u8()?);
    }
    Ok(result)
}

pub fn uniffi_write_bytes(
    cursor: &mut uniffi::FfiBufferCursor,
    value: Vec<u8>,
) -> uniffi::Result<()> {
    cursor.write_u64(value.len() as u64)?;
    for v in value {
        cursor.write_u8(v)?;
    }
    Ok(())
}
