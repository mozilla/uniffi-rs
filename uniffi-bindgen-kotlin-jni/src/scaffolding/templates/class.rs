{%- let type_name = cls.self_type.type_rs %}
{%- let arc_into_handle = "arc_into_handle_{}"|format(cls.self_type.id) %}
{%- let handle_into_arc = "handle_into_arc_{}"|format(cls.self_type.id) %}
{%- let lift_object_ref = "lift_object_ref_{}"|format(cls.self_type.id) %}
{%- let lift_object_receiver_ref = "lift_object_receiver_ref_{}"|format(cls.self_type.id) %}

{%- match cls.imp %}
{%- when ObjectImpl::Struct %}
unsafe fn {{ arc_into_handle }}(
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
) -> ::std::primitive::i64 {
    let raw_ptr = ::std::sync::Arc::into_raw(value.into_arc());
    uniffi::trace!("lower object: {raw_ptr:?}");
    raw_ptr as ::std::primitive::i64
}

unsafe fn {{ handle_into_arc }}(
    value: ::std::primitive::i64,
) -> {{ type_name }} {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(value as usize);
    uniffi::trace!("lift object: {raw_ptr:?}");
    ::std::sync::Arc::from_raw(raw_ptr)
}

/// Optimized lift function for references.
/// In this case, the Kotlin code doesn't clone a handle and we don't consume one.
unsafe fn {{ lift_object_ref }}<'a>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<&'a {{ cls.inner_type_name() }}> {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(value as usize);
    uniffi::trace!("lift ref: {raw_ptr:?}");
    uniffi::Result::Ok(&*raw_ptr)
}


{%- when ObjectImpl::Trait(trait_kind) %}
{%- if !trait_kind.has_foreign() %}
unsafe fn {{ arc_into_handle }}(
    value: {{ type_name }},
) -> ::std::primitive::i64 {
    // Wrap the Arc<dyn trait> into a second arc
    // this turns the wide pointer into a normal pointer which is easier to pass across
    // the FFI
    let value = ::std::sync::Arc::new(value);
    let raw_ptr = ::std::sync::Arc::into_raw(value);
    uniffi::trace!("lower object: {raw_ptr:?}");
    raw_ptr as ::std::primitive::i64
}

unsafe fn {{ handle_into_arc }}(
    value: ::std::primitive::i64,
) -> {{ type_name }} {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(value as usize);
    uniffi::trace!("lift object: {raw_ptr:?}");
    let arc = ::std::sync::Arc::from_raw(raw_ptr);
    // Deref to get the inner arc, then clone to return an owned copy
    (*arc).clone()
}

/// Optimized lift function for references.
/// In this case, the Kotlin code doesn't clone a handle and we don't consume one.
unsafe fn {{ lift_object_ref }}<'a>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<&'a dyn {{ cls.inner_type_name() }}> {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(value as usize);
    uniffi::trace!("lift ref: {raw_ptr:?}");
    // Note: extra deref, since there's an extra level of arc wrapping
    uniffi::Result::Ok(&**raw_ptr)
}

{%- else %}

unsafe fn {{ arc_into_handle }}(
    value: {{ type_name }},
) -> ::std::primitive::i64 {
    match value.uniffi_foreign_handle() {
        Some(handle) => {
            handle.as_raw() as ::std::primitive::i64
        }
        None => {
            // Wrap the Arc<dyn trait> into a second arc
            // this turns the wide pointer into a normal pointer which is easier to pass across
            // the FFI
            let value = ::std::sync::Arc::new(value);
            let raw_ptr = ::std::sync::Arc::into_raw(value);
            uniffi::trace!("lower object: {raw_ptr:?}");
            raw_ptr as ::std::primitive::i64
        }
    }
}

unsafe fn {{ handle_into_arc }}(
    value: ::std::primitive::i64,
) -> {{ type_name }} {
    if value & 1 == 1 {
        // Callback interface from Kotlin
        ::std::sync::Arc::new({{ cls.impl_struct_rs() }} {
            handle: value
        })
    } else {
        let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(value as usize);
        uniffi::trace!("lift object: {raw_ptr:?}");
        let arc = ::std::sync::Arc::from_raw(raw_ptr);
        // Deref to get the inner arc, then clone to return an owned copy
        (*arc).clone()
    }
}

unsafe fn {{ lift_object_receiver_ref }}<'a>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<&'a dyn {{ cls.inner_type_name() }}> {
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(value as usize);
    uniffi::trace!("lift ref: {raw_ptr:?}");
    // Note: extra deref, since there's an extra level of arc wrapping
    uniffi::Result::Ok(&**raw_ptr)
}

{%- endif %}
{%- endmatch %}

unsafe fn {{ cls.self_type.lower_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    {%- if cls.imp.is_trait_interface() %}
    value: ::std::sync::Arc<dyn {{ cls.inner_type_name() }}>,
    {%- else %}
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
    {%- endif %}
) -> uniffi::Result<::std::primitive::i64> {
    unsafe { uniffi::Result::Ok({{ arc_into_handle }}(value)) }
}

unsafe fn {{ cls.self_type.lift_fn_rs() }}(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: ::std::primitive::i64,
) -> uniffi::Result<{{ type_name }}> {
    unsafe { uniffi::Result::Ok({{ handle_into_arc }}(value)) }
}

unsafe fn {{ cls.self_type.write_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
    {%- if cls.imp.is_trait_interface() %}
    value: ::std::sync::Arc<dyn {{ cls.inner_type_name() }}>,
    {%- else %}
    value: impl uniffi::ArcOrOwned<{{ cls.inner_type_name() }}>,
    {%- endif %}
) -> uniffi::Result<()> {
    unsafe {
        let handle = {{ arc_into_handle }}(value);
        uniffi::ffibuffer::write_i64(ptr, handle)?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ cls.self_type.read_fn_rs() }}(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<{{ type_name }}> {
    unsafe {
        uniffi::Result::Ok({{ handle_into_arc }}(uniffi::ffibuffer::read_i64(ptr)?))
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_free_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) {
    {%- if !cls.imp.is_trait_interface() %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    {%- else %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(handle as usize);
    {%- endif %}
    uniffi::trace!("free object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        drop(::std::sync::Arc::from_raw(raw_ptr))
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_uniffi_Scaffolding_{{ cls.jni_clone_name() }}(
    _: *mut uniffi_jni::JNIEnv,
    _: *mut uniffi_jni::jclass,
    handle: i64,
) -> i64 {
    {%- if !cls.imp.is_trait_interface() %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<{{ cls.inner_type_name() }}>(handle as usize);
    {%- else %}
    let raw_ptr = ::std::ptr::with_exposed_provenance_mut::<::std::sync::Arc<dyn {{ cls.inner_type_name() }}>>(handle as usize);
    {%- endif %}
    uniffi::trace!("clone object: {raw_ptr:?}");
    // Safety:
    // raw_ptr came from an `into_raw()` call
    unsafe {
        ::std::sync::Arc::increment_strong_count(raw_ptr);
    }
    // Return the handle back now that the ref count has been incremented.
    handle
}
