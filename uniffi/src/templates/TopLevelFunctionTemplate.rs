{#
// For each top-level function declared in the IDL, we assume the caller has provided a corresponding
// rust function of the same name. We provide a `pub extern "C"` wrapper that does type conversions to
// send data across the FFI, which will fail to compile if the provided function does not match what's
// specified in the IDL.    
#}
{%- match func.ffi_func().return_type() -%}
{%- when Some with (return_type) %}

#[no_mangle]
pub extern "C" fn {{ func.ffi_func().name() }}(
    {%- call rs::arg_list_rs_decl(func.ffi_func().arguments()) %}) -> {{ return_type|ret_type_c }} {
    log::debug!("{{ func.ffi_func().name() }}");
    let _retval = {% call rs::to_rs_call(func) %};
    {{ "_retval"|lower_rs(return_type) }}
}

{% when None -%}

#[no_mangle]
pub extern "C" fn {{ func.ffi_func().name() }}(
    {%- call rs::arg_list_rs_decl(func.ffi_func().arguments()) %}) {
    log::debug!("{{ func.ffi_func().name() }}");
    {% call rs::to_rs_call(func) %};
}
{% endmatch %}