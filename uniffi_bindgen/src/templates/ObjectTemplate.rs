// For each Object definition, we assume the caller has provided an appropriately-shaped `struct`
// with an `impl` for each method on the object. We create a `ConcurrentHandleMap` for safely handing
// out references to these structs to foreign language code, and we provide a `pub extern "C"` function
// corresponding to each method.
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the UDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.
{% let handle_map = format!("UNIFFI_HANDLE_MAP_{}", obj.name().to_uppercase()) -%}

uniffi::deps::lazy_static::lazy_static! {
    {%- let handle_map_type = obj.threadsafe()|choose(
        "uniffi::ffi::handle_maps::ArcHandleMap",
        "uniffi::ffi::handle_maps::MutexHandleMap")
    %}
    #[doc(hidden)]
    static ref {{ handle_map }}: {{ handle_map_type }}<{{ obj.name() }}>
        = Default::default();
}

    {% let ffi_free = obj.ffi_object_free() -%}
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn {{ ffi_free.name() }}(handle: u64) {
        let _ = {{ handle_map }}.delete_u64(handle);
    }

{%- for cons in obj.constructors() %}
    #[allow(clippy::all)]
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn {{ cons.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(cons.ffi_func()) %}) -> u64 {
        uniffi::deps::log::debug!("{{ cons.ffi_func().name() }}");
        // If the constructor does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_constructor_call(obj, cons) %}
    }
{%- endfor %}

{%- for meth in obj.methods() %}
    #[allow(clippy::all)]
    #[doc(hidden)]
    #[no_mangle]
    pub extern "C" fn {{ meth.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(meth.ffi_func()) %}
    ) -> {% call rs::return_type_func(meth) %} {
        uniffi::deps::log::debug!("{{ meth.ffi_func().name() }}");
        // If the method does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% if meth.is_static() %}
        {% call rs::to_rs_static_method_call(obj, meth) %}
        {% else %}
        {% call rs::to_rs_method_call(obj, meth) %}
        {% endif %}
    }
{% endfor %}

// We proide a `ViaFfi` impl for each object, but the only thing it can currently
// do is lower an owned instance into a handle. This is useful for returning new
// instances of the object from static methods, and we know it is safe and sound
// because Rust's ownership system ensures that:
//  * the thing we're operating on is an owned instance (not a reference) and
//    so we know that nothing else in the program is looking at it.
//  * the ownership is irrevokably transferred to the handlemap, and hence to
//    the foreign language code, which takes responsibility for it from there.
//
// Other operations are not yet supported, since they would involve much more
// complicated semantics around references.
unsafe impl uniffi::ViaFfi for {{ obj.name() }} {
    type FfiType = u64;

    fn lower(self) -> Self::FfiType {
        // Note that this consumes `self`, transferring ownership to the handlemap.
        {{ handle_map }}.insert(self).into_u64()
    }

    fn try_lift(_v: Self::FfiType) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::deps::anyhow::bail!("Lifting object types is not yet supported");
    }

    fn write<B: uniffi::deps::bytes::BufMut>(&self, _buf: &mut B) {
        panic!("Writing object types is not yet supported");
    }

    fn try_read<B: uniffi::deps::bytes::Buf>(_buf: &mut B) -> uniffi::deps::anyhow::Result<Self> {
        uniffi::deps::anyhow::bail!("Reading object types is not yet supported");
    }
}