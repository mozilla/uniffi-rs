{%- let cbi = ci|get_callback_interface_definition(name) %}
{%- let callback_handler_class = format!("UniffiCallbackInterface{}", name) %}
{%- let callback_handler_obj = format!("uniffiCallbackInterface{}", name) %}
{%- let ffi_init_callback = cbi.ffi_init_callback() %}
{%- let protocol_name = type_name.clone() %}
{%- let methods = cbi.methods() %}
{%- let protocol_docstring = cbi.docstring() %}

{% include "Protocol.py" %}
{% include "CallbackInterfaceImpl.py" %}

# The _UniffiConverter which transforms the Callbacks in to Handles to pass to Rust.
{{ ffi_converter_name }} = UniffiCallbackInterfaceFfiConverter()
