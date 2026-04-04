#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferCheckSupport(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
) {
    // Try to create a direct byte buffer with an arbitrary address.
    // If this returns null, then the JVM doesn't support JNI direct buffer access
    unsafe {
        let ptr = ((**env).v1_4.NewDirectByteBuffer)(env, ::std::ptr::NonNull::dangling().as_ptr(), 1);
        if ptr.is_null() {
            uniffi_jni::throw_internal_exception(env, uniffi_jni::JniString::from("nio DirectBuffers not supported".to_string()));
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferAlloc(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    capacity: ::std::primitive::i32
) -> uniffi_jni::jobject {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let ptr = uniffi::ffibuffer::alloc(capacity as ::std::primitive::usize)?;
            uniffi::Result::Ok(((**env).v1_4.NewDirectByteBuffer)(env, ptr.cast(), capacity as ::std::primitive::i64))
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferFree(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf: uniffi_jni::jobject,
) {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let (ptr, capacity) = uniffi_jni::lift_buffer(env, buf)?;
            uniffi::ffibuffer::free(ptr, capacity);
            uniffi::Result::Ok(())
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferReadString(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf: uniffi_jni::jobject,
    offset: ::std::primitive::i32,
) -> uniffi_jni::jstring {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let (ptr, capacity) = uniffi_jni::lift_buffer(env, buf)?;
            let ptr = ptr.add(offset as ::std::primitive::usize);
            let s = uniffi::ffibuffer::read_string(ptr)?;
            uniffi_jni::lower_string(env, s)
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferWriteString(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf: uniffi_jni::jobject,
    offset: ::std::primitive::i32,
    value: uniffi_jni::jstring,
) {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let s = uniffi_jni::lift_string(env, value)?;
            let (ptr, _) = uniffi_jni::lift_buffer(env, buf)?;
            let ptr = ptr.add(offset as ::std::primitive::usize);
            uniffi::ffibuffer::write_string(ptr, s)?;
            uniffi::Result::Ok(())
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferReadBuffer(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf: uniffi_jni::jobject,
    offset: ::std::primitive::i32,
) -> uniffi_jni::jobject {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let (ptr, _) = uniffi_jni::lift_buffer(env, buf)?;
            let ptr = ptr.add(offset as ::std::primitive::usize);
            let (child_ptr, child_capacity) = uniffi::ffibuffer::read_buffer(ptr)?;
            uniffi_jni::lower_buffer(env, child_ptr, child_capacity)
        })
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_ffiBufferWriteBuffer(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf: uniffi_jni::jobject,
    offset: ::std::primitive::i32,
    child_buf: uniffi_jni::jobject,
) {
    unsafe {
        uniffi_jni::rust_call_with_env(env, |env| {
            let (ptr, _) = uniffi_jni::lift_buffer(env, buf)?;
            let ptr = ptr.add(offset as ::std::primitive::usize);
            let (child_ptr, child_capacity) = uniffi_jni::lift_buffer(env, child_buf)?;
            uniffi::ffibuffer::write_buffer(ptr, child_ptr, child_capacity)?;
            uniffi::Result::Ok(())
        })
    }
}
