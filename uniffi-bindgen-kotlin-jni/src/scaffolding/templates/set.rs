{#-
  These functions are generic over the container: hand-written FFI impls can expose
  third-party containers (e.g. `indexmap::IndexSet`) with builtin container metadata,
  so the Rust type at a use site isn't necessarily the std container.  Call sites
  always have a concrete type for inference: a record field, a function argument, or
  a function return value.
-#}
{%- let type_name = set.self_type.type_rs %}
{%- let item_type = set.inner.type_rs %}
{%- let lift_from_parts = "lift_from_parts_{}"|format(set.self_type.id) %}
{%- let lower_into_parts = "lower_into_parts_{}"|format(set.self_type.id) %}

unsafe fn {{ lower_into_parts }}<C>(
    value: C,
) -> uniffi::Result<(*mut ::std::primitive::u8, ::std::primitive::usize)>
where
    C: ::std::iter::IntoIterator<Item = {{ item_type }}>,
    C::IntoIter: ::std::iter::ExactSizeIterator,
{
    unsafe {
        let items = value.into_iter();
        let capacity = items.len() * {{ set.item_size }};
        let ptr = uniffi::ffibuffer::alloc(capacity)?;
        let mut pos = ptr;
        for v in items {
            {{ set.inner.write_fn_rs() }}(pos, v)?;
            pos = pos.add({{ set.item_size }});
        }
        uniffi::Result::Ok((ptr, capacity))
    }
}

unsafe fn {{ lift_from_parts }}<C>(
    ptr: *mut ::std::primitive::u8,
    capacity: ::std::primitive::usize,
) -> uniffi::Result<C>
where
    C: ::std::iter::FromIterator<{{ item_type }}>,
{
    let mut do_lift = || {
        let length = capacity / {{ set.item_size }};
        let mut pos = ptr;
        (0..length)
            .map(|_| {
                let item = unsafe { {{ set.inner.read_fn_rs() }}(pos) }?;
                pos = unsafe { pos.add({{ set.item_size }}) };
                uniffi::Result::Ok(item)
            })
            .collect::<uniffi::Result<C>>()
    };
    let result = do_lift();
    unsafe { uniffi::ffibuffer::free(ptr, capacity) };
    result
}

unsafe fn {{ set.self_type.lower_fn_rs() }}<C>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    value: C,
) -> uniffi::Result<uniffi_jni::jobject>
where
    C: ::std::iter::IntoIterator<Item = {{ item_type }}>,
    C::IntoIter: ::std::iter::ExactSizeIterator,
{
    unsafe {
        let (ptr, capacity) = {{ lower_into_parts }}(value)?;
        uniffi_jni::lower_buffer(uniffi_env, ptr, capacity)
    }
}

unsafe fn {{ set.self_type.lift_fn_rs() }}<C>(
    uniffi_env: *mut uniffi_jni::JNIEnv,
    byte_buffer: uniffi_jni::jobject,
) -> uniffi::Result<C>
where
    C: ::std::iter::FromIterator<{{ item_type }}>,
{
    unsafe {
        let (ptr, capacity) = uniffi_jni::lift_buffer(uniffi_env, byte_buffer)?;
        {{ lift_from_parts }}(ptr, capacity)
    }
}

unsafe fn {{ set.self_type.write_fn_rs() }}<C>(
    buf_ptr: *mut ::std::primitive::u8,
    value: C,
) -> uniffi::Result<()>
where
    C: ::std::iter::IntoIterator<Item = {{ item_type }}>,
    C::IntoIter: ::std::iter::ExactSizeIterator,
{
    unsafe {
        let (ptr, capacity) = {{ lower_into_parts }}(value)?;
        uniffi::ffibuffer::write_buffer(buf_ptr, ptr, capacity)?;
        uniffi::Result::Ok(())
    }
}

unsafe fn {{ set.self_type.read_fn_rs() }}<C>(
    ptr: *mut ::std::primitive::u8,
) -> uniffi::Result<C>
where
    C: ::std::iter::FromIterator<{{ item_type }}>,
{
    unsafe {
        let (ptr, capacity) = uniffi::ffibuffer::read_buffer(ptr)?;
        {{ lift_from_parts }}(ptr, capacity)
    }
}
