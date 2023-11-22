{%- let cbi = ci|get_callback_interface_definition(name) %}
{%- let callback_handler_class = format!("UniffiCallbackInterface{}", name) %}
{%- let callback_handler_obj = format!("uniffiCallbackInterface{}", name) %}
{%- let ffi_init_callback = cbi.ffi_init_callback() %}
{%- let interface_name = cbi|type_name(ci) %}
{%- let methods = cbi.methods() %}
{%- let interface_docstring = cbi.docstring() %}

{% include "Interface.kt" %}
{% include "CallbackInterfaceImpl.kt" %}

// The ffiConverter which transforms the Callbacks in to Handles to pass to Rust.
public object {{ ffi_converter_name }}: FfiConverterCallbackInterface<{{ interface_name }}>()
