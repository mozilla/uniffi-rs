# This is how we find and load the dynamic library provided by the component.
# For now we just look it up by name.
module UniFFILib
  extend FFI::Library

  {% if config.custom_cdylib_path() %}
  ffi_lib {{ config.cdylib_path() }}
  {% else %}
  ffi_lib '{{ config.cdylib_name() }}'
  {% endif %}

  # Define FFI callback types and structs (vtables, etc.)
  {% for def in ci.ffi_definitions() %}
  {%- match def -%}

  {%- when FfiDefinition::CallbackFunction(cb_fn) %}
  callback :{{ cb_fn.name() }},
    [{%- for arg in cb_fn.arguments() %}{{ arg.type_().borrow()|type_ffi }}, {% endfor -%}
    {%- if cb_fn.has_rust_call_status_arg() -%}RustCallStatus.by_ref{% endif -%}],
    {% match cb_fn.return_type() %}{% when Some with (type_) %}{{ type_|type_ffi }}{% when None %}:void{% endmatch %}

  {%- when FfiDefinition::Struct(ffi_struct) %}
  class {{ ffi_struct.name() }} < FFI::Struct
    layout(
      {%- for field in ffi_struct.fields() %}
      :{{ field.name() }}, {{ field.type_().borrow()|type_ffi }}{% if !loop.last %},{% endif %}
      {%- endfor %}
    )
  end

  {%- when FfiDefinition::Function(func) %}
  attach_function :{{ func.name() }},
    {%- call rb::arg_list_ffi_decl(func) %}{% endcall %},
    {% match func.return_type() %}{% when Some with (type_) %}{{ type_|type_ffi }}{% when None %}:void{% endmatch %}
  {%- endmatch %}
  {% endfor %}
end
