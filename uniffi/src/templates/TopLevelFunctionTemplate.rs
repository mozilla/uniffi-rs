{#
// For each top-level function declared in the IDL, we assume the caller has provided a corresponding
// rust function of the same name. We provide a `pub extern "C"` wrapper that does type conversions to
// send data across the FFI, which will fail to compile if the provided function does not match what's
// specified in the IDL.    
#}
#[no_mangle]
pub extern "C" fn {{ func.ffi_func().name() }}(
    {%- for arg in func.ffi_func().arguments() %}
    {{ arg.name() }}: {{ arg.type_()|type_c }},
    {%- endfor %}
) -> {% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|ret_type_c }}{% else %}(){% endmatch %} {
    log::debug!("{{ func.ffi_func().name() }}");
    // If the provided function does not match the signature specified in the IDL
    // then this attempt to cal it will not compile, and will give guideance as to why.
    let _retval = {{ func.name() }}(
        {%- for arg in func.arguments() %}
        {{ arg.name()|lift_rs(arg.type_()) }},
        {%- endfor %}
    );
    {% match func.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lower_rs(return_type) }}{% else %}{% endmatch %}
}