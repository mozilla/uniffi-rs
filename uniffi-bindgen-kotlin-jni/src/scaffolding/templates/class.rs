{%- let type_name = cls.self_type.type_rs %}
{%- let inner_type_name = "{}::{}"|format(cls.module_path, cls.name_rs()) %}

{%- match cls.imp %}
{%- when ObjectImpl::Struct %}
/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ cls.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<{{ type_name }}> {
    let raw_ptr = cursor.read_ptr::<{{ inner_type_name }}>()?;
    uniffi::trace!("{{ cls.name }} read: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        Ok(::std::sync::Arc::from_raw(raw_ptr))
    }
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
///
/// Inputs ArcOrOwned<T> so that it's compatible with both `T` and `Arc<T>`.
pub fn {{ cls.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: impl uniffi::ArcOrOwned<{{ inner_type_name }}>,
) -> uniffi::Result<()> {
    let raw_ptr = ::std::sync::Arc::into_raw(value.into_arc());
    uniffi::trace!("{{ cls.name }} write: {raw_ptr:?}");
    cursor.write_ptr::<{{ inner_type_name }}>(raw_ptr.cast_mut())?;
    Ok(())
}
{%- when ObjectImpl::Trait %}
/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ cls.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<::std::sync::Arc<dyn {{ inner_type_name }}>> {
    let raw_ptr1 = cursor.read_ptr::<()>()?;
    let raw_ptr2 = cursor.read_ptr::<()>()?;
    uniffi::trace!("{{ cls.name }} read: {raw_ptr1:?} {raw_ptr2:?}");
    // Safety:
    // This is reversing a transmute/into_raw by {{ cls.self_type.write_fn_rs }}
    unsafe {
        let raw_ptr: *const dyn {{ inner_type_name }} = ::std::mem::transmute([raw_ptr1, raw_ptr2]);
        Ok(::std::sync::Arc::from_raw(raw_ptr))
    }
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
///
/// Inputs ArcOrOwned<T> so that it's compatible with both `T` and `Arc<T>`.
pub fn {{ cls.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: ::std::sync::Arc<dyn {{ inner_type_name }}>,
) -> uniffi::Result<()> {
    let raw_ptr = ::std::sync::Arc::into_raw(value);

    // Safety:
    // A wide pointer has the same layout as 2 normal pointers
    let [raw_ptr1, raw_ptr2]: [*mut (); 2] = unsafe {
        std::mem::transmute(raw_ptr)
    };
    uniffi::trace!("{{ cls.name }} write: {raw_ptr1:?} {raw_ptr2:?}");
    cursor.write_ptr::<()>(raw_ptr1)?;
    cursor.write_ptr::<()>(raw_ptr2)?;
    Ok(())
}

{%- when ObjectImpl::CallbackTrait %}
/// Read a {{ type_name }} from a `FfiBufferCursor`
pub fn {{ cls.self_type.read_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
) -> uniffi::Result<::std::sync::Arc<dyn {{ inner_type_name }}>> {
    let handle1 = cursor.read_i64()?;
    let handle2 = cursor.read_i64()?;
    uniffi::trace!("{{ cls.name }} read: {handle1} {handle2}");

    if handle1 == 0 {
        // Callback interface from Kotlin
        Ok(::std::sync::Arc::new({{ cls.impl_struct_rs() }} {
            handle: handle2
        }))
    } else {
        // Arc<dyn Trait> impl from Rust
        // Safety:
        // This is reversing a transmute/into_raw by {{ cls.self_type.write_fn_rs }}
        unsafe {
            let raw_ptr1 = ::std::ptr::with_exposed_provenance::<()>(handle1 as usize);
            let raw_ptr2 = ::std::ptr::with_exposed_provenance::<()>(handle2 as usize);
            let raw_ptr: *const dyn {{ inner_type_name }} = ::std::mem::transmute([raw_ptr1, raw_ptr2]);
            Ok(::std::sync::Arc::from_raw(raw_ptr))
        }
    }
}

/// Write a {{ type_name }} to a `FfiBufferCursor`
///
/// Inputs ArcOrOwned<T> so that it's compatible with both `T` and `Arc<T>`.
pub fn {{ cls.self_type.write_fn_rs }}(
    cursor: &mut uniffi::FfiBufferCursor,
    value: ::std::sync::Arc<dyn {{ inner_type_name }}>,
) -> uniffi::Result<()> {
    match value.uniffi_foreign_handle() {
        Some(handle) => {
            uniffi::trace!("{{ cls.name }} write foreign: {handle:?}");
            cursor.write_i64(0)?;
            cursor.write_i64(handle.as_raw() as i64)?;
        }
        None => {
            let raw_ptr = ::std::sync::Arc::into_raw(value);
            uniffi::trace!("{{ cls.name }} write rust: {raw_ptr:?}");
            // Safety:
            // A wide pointer has the same layout as 2 normal pointers
            let [raw_ptr1, raw_ptr2]: [*mut (); 2] = unsafe {
                std::mem::transmute(raw_ptr)
            };
            cursor.write_i64(raw_ptr1.expose_provenance() as i64)?;
            cursor.write_i64(raw_ptr2.expose_provenance() as i64)?;
        }
    }
    Ok(())
}
{%- endmatch %}

{%- if !cls.imp.is_trait_interface() %}
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ inner_type_name }}>(handle as usize);
    uniffi::trace!("{{ cls.name }} free: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_addref_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ inner_type_name }}>(handle as usize);
    uniffi::trace!("{{ cls.name }} addref: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
}
{%- else %}
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle1: i64,
    handle2: i64,
) {
    let raw_ptr1 = ::std::ptr::with_exposed_provenance::<()>(handle1 as usize);
    let raw_ptr2 = ::std::ptr::with_exposed_provenance::<()>(handle2 as usize);
    uniffi::trace!("{{ cls.name }} free: {raw_ptr1:?} {raw_ptr2:?}");
    // Safety:
    // This is reversing a transmute/into_raw by {{ cls.self_type.write_fn_rs }}
    unsafe {
        let raw_ptr: *const dyn {{ inner_type_name }} = ::std::mem::transmute([raw_ptr1, raw_ptr2]);
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_addref_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle1: i64,
    handle2: i64,
) {
    let raw_ptr1 = ::std::ptr::with_exposed_provenance::<()>(handle1 as usize);
    let raw_ptr2 = ::std::ptr::with_exposed_provenance::<()>(handle2 as usize);
    uniffi::trace!("{{ cls.name }} addref: {raw_ptr1:?} {raw_ptr2:?}");
    // Safety:
    // This is reversing a transmute/into_raw by {{ cls.self_type.write_fn_rs }}
    unsafe {
        let raw_ptr: *const dyn {{ inner_type_name }} = ::std::mem::transmute([raw_ptr1, raw_ptr2]);
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
}
{%- endif %}
