// For each Object definition, we assume the caller has provided an appropriately-shaped `struct`
// with an `impl` for each method on the object. We create a `ConcurrentHandleMap` for safely handing
// out references to these structs to foreign language code, and we provide a `pub extern "C"` function
// corresponding to each method.
//
// If the caller's implementation of the struct does not match with the methods or types specified
// in the IDL, then the rust compiler will complain with a (hopefully at least somewhat helpful!)
// error message when processing this generated code.

lazy_static::lazy_static! {
    static ref UNIFFI_HANDLE_MAP_{{ obj.name()|upper }}: ffi_support::ConcurrentHandleMap<{{ obj.name() }}> = ffi_support::ConcurrentHandleMap::new();
}

// XXX TODO: destructors.
// These will need to be defined as another FFI function on the Object struct, and included automatically
// in the set of all FFI functions for use by the bindings.
// define_handle_map_deleter!(UNIFFI_HANDLE_MAP_{{ obj.name() }}, {{ obj.name() }}_free);

{%- for cons in obj.constructors() %}
    #[no_mangle]
    pub extern "C" fn {{ cons.ffi_func().name() }}(
        {%- call rs::arg_list_rs_decl(cons.ffi_func()) %}) -> u64 {
        log::debug!("{{ cons.ffi_func().name() }}");
        // If the constructor does not have the same signature as declared in the IDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_constructor_call(obj, cons) %}
    }
{% endfor %}

{%- for meth in obj.methods() %}
#[no_mangle]
    pub extern "C" fn {{ meth.ffi_func().name() }}(
        {%- call rs::arg_list_rs_decl(meth.ffi_func()) %}
    ) -> {% call rs::return_type_func(meth) %} {
        log::debug!("{{ meth.ffi_func().name() }}");
        // If the method does not have the same signature as declared in the IDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        {% call rs::to_rs_method_call(obj, meth) %}
    }
{% endfor %}
