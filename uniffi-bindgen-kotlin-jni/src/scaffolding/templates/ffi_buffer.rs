#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_ffiBufferNew(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    size: i64,
) -> i64 {
    uniffi::ffi_buffer_alloc().expose_provenance() as i64
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_miniBufferNext(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    end: i64,
    size: i64,
) -> i64 {
    let end = ::std::ptr::with_exposed_provenance_mut(end as usize);
    let size = size as usize;
    // Safety:
    // end is pointed to the end of the mini buffer
    let next_ptr = unsafe { uniffi::mini_buffer_next(end, size) };
    next_ptr.expose_provenance() as i64
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_ffiBufferFree(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    // Safety:
    // We assume the other side of the FFI sent us a valid handle
    unsafe {
        uniffi::ffi_buffer_free(::std::ptr::with_exposed_provenance_mut(handle as usize))
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readByte(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> i8 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<i8>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readShort(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> i16 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<i16>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readInt(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> i32 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<i32>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readLong(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> i64 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<i64>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readFloat(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> f32 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<f32>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readDouble(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> f64 {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance::<f64>(ptr as usize).read()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_readString(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
) -> uniffi_jni::jstring {
    let buf_ptr = ::std::ptr::with_exposed_provenance_mut(ptr as usize);
    // Safety:
    //
    // * env points to a valid JNIEnv
    // * We assume the other side of the FFI passed us a valid pointer with valid string parts.
    unsafe {
        uniffi_jni::rust_call(env, |env| {
            let value = uniffi::read_string_from_pointer(buf_ptr)?;
            let value = uniffi_jni::JniString::from(value);
            Ok(value.into_jstring(env))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeByte(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: i8,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<i8>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeShort(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: i16,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<i16>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeInt(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: i32,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<i32>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeLong(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: i64,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<i64>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeFloat(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: f32,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<f32>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeDouble(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    ptr: i64,
    value: f64,
) {
    // Safety:
    // We assume the other side of the FFI gave us a valid address
    unsafe {
        ::std::ptr::with_exposed_provenance_mut::<f64>(ptr as usize).write(value)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_uniffi_Scaffolding_writeString(
    env: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    buf_ptr: i64,
    value: uniffi_jni::jstring,
) {
    let buf_ptr = ::std::ptr::with_exposed_provenance_mut(buf_ptr as usize);
    // Safety:
    //
    // * env points to a valid JNIEnv
    // * We assume the Kotlin side of the FFI passed us a `buf_ptr` pointing at the correct
    //   location.
    unsafe {
        uniffi_jni::rust_call(env, |env| {
            let value = uniffi_jni::decode_jni_string(env, value)?;
            uniffi::write_string_to_pointer(buf_ptr, value);
            Ok(())
        })
    }
}
