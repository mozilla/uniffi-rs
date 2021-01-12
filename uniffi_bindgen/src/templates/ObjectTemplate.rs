// For each Object definition, we assume the caller has provided an appropriately-shaped `struct`
// with an `impl` for each method on the object. We generate some boxing code in order to represent
// instances of this struct as raw pointers in the FFI, and some static assertions to ensure this
// is safe (well...TBD exactly the circumstances under which this is safe, but it's a start).
// We provide a `pub extern "C"` function corresponding to each method on the object, which takes
// an instance pointer as first argument.
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the UDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.

// Pass objects across the FFI as pointers, in using a box.
//
// Per the `Box` docs:
// "So long as T: Sized, a Box<T> is guaranteed to be represented as a single pointer and is also ABI-compatible with C pointers".
//
// Note that implementing `IntoFfi` for `T` asserts that `T: Sized`.

unsafe impl uniffi::deps::ffi_support::IntoFfi for {{ obj.name() }} {
    type Value = Option<Box<{{ obj.name() }}>>;
    fn ffi_default() -> Self::Value { None }
    fn into_ffi_value(self) -> Self::Value { Some(Box::new(self)) }
}

// For thread-safety, we only support raw pointers on things that are Sync and Send.
uniffi::deps::static_assertions::assert_impl_all!({{ obj.name() }}: Sync, Send);

#[no_mangle]
pub extern "C" fn {{ obj.ffi_object_free().name() }}(obj : Option<Box<{{ obj.name() }}>>) {
    drop(obj);
}

{%- for cons in obj.constructors() %}
    #[allow(clippy::all)]
    #[no_mangle]
    pub extern "C" fn {{ cons.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(cons.ffi_func()) %}
    ) -> Option<Box<{{ obj.name() }}>> {
        uniffi::deps::log::debug!("{{ cons.ffi_func().name() }} - raw pointer version");
        // If the constructor does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_constructor_call(obj, cons) %}
    }
{%- endfor %}

{%- for meth in obj.methods() %}
    #[allow(clippy::all)]
    #[no_mangle]
    pub extern "C" fn {{ meth.ffi_func().name() }}(
        {%- call rs::arg_list_ffi_decl(meth.ffi_func()) %}
    ) -> {% call rs::return_type_func(meth) %} {
        uniffi::deps::log::debug!("{{ meth.ffi_func().name() }} - raw pointer version");
        // If the method does not have the same signature as declared in the UDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_method_call(obj, meth) %}
    }
{% endfor %}
