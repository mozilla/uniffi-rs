unsafe fn {{ type_node.lower_fn_rs() }}(
    _: *mut uniffi_jni::JNIEnv,
    value: ::std::time::Duration,
) -> uniffi::Result<(::std::primitive::i64, ::std::primitive::i32)> {
    Ok((value.as_secs() as ::std::primitive::i64, value.subsec_nanos() as ::std::primitive::i32))
}

unsafe fn {{ type_node.lift_fn_rs() }}(
    _: *mut uniffi_jni::JNIEnv,
    seconds: ::std::primitive::i64,
    nanos: ::std::primitive::i32,
) -> uniffi::Result<::std::time::Duration> {
    Ok(::std::time::Duration::new(seconds as ::std::primitive::u64, nanos as ::std::primitive::u32))
}

unsafe fn {{ type_node.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    value: ::std::time::Duration,
) -> uniffi::Result<()> {
    unsafe {
        uniffi::ffibuffer::write_u64(ptr, value.as_secs())?;
        uniffi::ffibuffer::write_u32(ptr.add(8), value.subsec_nanos())?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ type_node.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<::std::time::Duration> {
    unsafe {
        Ok(::std::time::Duration::new(
            uniffi::ffibuffer::read_u64(ptr)?,
            uniffi::ffibuffer::read_u32(ptr.add(8))?,
        ))
    }
}
