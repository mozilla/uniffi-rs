{%- let vtable = cbi.vtable %}
{% include "VTable.py" %}

# The _UniffiConverter which transforms the Callbacks in to Handles to pass to Rust.
{{ ffi_converter_name }} = _UniffiCallbackInterfaceFfiConverter()
