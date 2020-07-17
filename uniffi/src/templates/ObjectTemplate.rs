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
        {%- for arg in cons.ffi_func().arguments() %}
        {{ arg.name() }}: {{ arg.type_()|type_c }},
        {%- endfor %}
    ) -> u64 {
        log::debug!("{{ cons.ffi_func().name() }}");
        let mut err: ffi_support::ExternError = Default::default(); // XXX TODO: error handling!
        // If the constructor does not have the same signature as declared in the IDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        let _handle = UNIFFI_HANDLE_MAP_{{ obj.name()|upper }}.insert_with_output(&mut err, || {
            let obj = {{ obj.name() }}::{{ cons.name() }}(
                {%- for arg in cons.arguments() %}
                {{ arg.name()|lift_rs(arg.type_()) }},
                {%- endfor %}
            );
            obj
        });
        _handle
    }
{% endfor %}

{%- for meth in obj.methods() %}
    #[no_mangle]
    pub extern "C" fn {{ meth.ffi_func().name() }}(
        {%- for arg in meth.ffi_func().arguments() %}
        {{ arg.name() }}: {{ arg.type_()|type_c }},
        {%- endfor %}
    ) -> {% match meth.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|type_c }}{% else %}(){% endmatch %} {
        log::debug!("{{ meth.ffi_func().name() }}");
        let mut err: ffi_support::ExternError = Default::default(); // XXX TODO: error handling!
        // If the method does not have the same signature as declared in the IDL, then
        // this attempt to call it will fail with a (somewhat) helpful compiler error.
        UNIFFI_HANDLE_MAP_{{ obj.name()|upper }}.call_with_output_mut(&mut err, {{ meth.first_argument().name() }}, |obj| {
            let _retval = {{ obj.name() }}::{{ meth.name() }}(
                obj,
                {%- for arg in meth.arguments() %}
                {{ arg.name()|lift_rs(arg.type_()) }},
                {%- endfor %}
            );
            {% match meth.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lower_rs(return_type) }}{% else %}{% endmatch %}
        })
    }
{% endfor %}

