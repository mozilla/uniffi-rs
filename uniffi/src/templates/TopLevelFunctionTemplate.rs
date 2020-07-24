{#
// For each top-level function declared in the IDL, we assume the caller has provided a corresponding
// rust function of the same name. We provide a `pub extern "C"` wrapper that does type conversions to
// send data across the FFI, which will fail to compile if the provided function does not match what's
// specified in the IDL.    
#}

#[no_mangle]
pub extern "C" fn {{ func.ffi_func().name() }}(
    {% call rs::arg_list_rs_decl(func.ffi_func()) %}
) -> {% call rs::return_type_func(func) %} {
    // If the provided function does not match the signature specified in the IDL
    // then this attempt to cal it will not compile, and will give guideance as to why.
    log::debug!("{{ func.ffi_func().name() }}");
    {% call rs::to_rs_function_call(func) %}
}
