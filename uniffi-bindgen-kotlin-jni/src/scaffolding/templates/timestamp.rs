fn system_time_to_parts(value: ::std::time::SystemTime) -> (::std::primitive::i64, ::std::primitive::u32) {
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

    (seconds, epoch_offset.subsec_nanos())
}

fn system_time_from_parts(
    seconds: ::std::primitive::i64,
    nanos: ::std::primitive::u32,
) -> ::std::time::SystemTime {
    let epoch_offset = ::std::time::Duration::new(seconds.wrapping_abs() as ::std::primitive::u64, nanos);

    if seconds >= 0 {
        ::std::time::SystemTime::UNIX_EPOCH + epoch_offset
    } else {
        ::std::time::SystemTime::UNIX_EPOCH - epoch_offset
    }
}

unsafe fn {{ type_node.lower_fn_rs() }}(
    _: *mut uniffi_jni::JNIEnv,
    v: ::std::time::SystemTime,
) -> uniffi::Result<(::std::primitive::i64, ::std::primitive::i32)> {
    let (seconds, nanos) = system_time_to_parts(v);
    Ok((
        seconds,
        nanos as ::std::primitive::i32,
    ))
}

unsafe fn {{ type_node.lift_fn_rs() }}(
    _: *mut uniffi_jni::JNIEnv,
    seconds: ::std::primitive::i64,
    nanos: ::std::primitive::i32,
) -> uniffi::Result<::std::time::SystemTime> {
    Ok(system_time_from_parts(seconds, nanos as ::std::primitive::u32))
}

unsafe fn {{ type_node.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: ::std::time::SystemTime,
) -> uniffi::Result<()> {
    unsafe {
        let (seconds, nanos) = system_time_to_parts(value);
        uniffi::ffibuffer::write_i64(ptr, seconds)?;
        uniffi::ffibuffer::write_u32(ptr.add(8), nanos)?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ type_node.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<::std::time::SystemTime> {
    unsafe {
        Ok(system_time_from_parts(
            uniffi::ffibuffer::read_i64(ptr)?,
            uniffi::ffibuffer::read_u32(ptr.add(8))?,
        ))
    }
}
