{%- let protocol = cbi.protocol %}
{%- let vtable = cbi.vtable %}
{%- let trait_impl=format!("_UniffiTraitImpl{}", cbi.name) %}
{%- let ffi_converter_name = cbi.self_type.ffi_converter_name %}

{% include "Protocol.py" %}
{% include "CallbackInterfaceImpl.py" %}

# The _UniffiConverter which transforms the Callbacks in to Handles to pass to Rust.
{{ ffi_converter_name }} = _UniffiCallbackInterfaceFfiConverter()
