{#
// Template to generate a vtable implementation for a callback interface.
// This template is used by the `CallbackInterfaceTemplate.rb` (for pure callback interfaces)
// and `ObjectTemplate.rb` (for trait interfaces with ObjectImpl::CallbackTrait).
//
// Expected variable: cbi_name (the callback interface name string)
#}
{%- let cbi = ci.get_callback_interface_definition(cbi_name).unwrap() %}
{%- let vtable_def = cbi.vtable_definition() %}
{%- let vtable_methods = cbi.vtable_methods() %}
{%- let ffi_init_callback = cbi.ffi_init_callback() %}

# Callback interface vtable implementation for {{ cbi_name }}.
module UniffiCallbackInterface{{ cbi_name|class_name_rb }}

  {%- for (ffi_callback, method)  in vtable_methods.iter() %}

  {%- if method.is_async() %}
  # Async callback method for {{ method.name() }}
  {{ method.name()|enum_name_rb }}_CALLBACK = Proc.new do |uniffi_handle, {%- for arg in method.arguments() %} {{ arg.name()|var_name_rb }},{% endfor %} uniffi_future_callback, uniffi_callback_data, uniffi_out_dropped_callback|
    uniffi_obj = CallbackInterface{{ cbi_name|class_name_rb }}FfiConverter.lift uniffi_handle
    {%- call rb::make_call_proc(method) %}{% endcall %}
    {%- call rb::async_handle_success_proc(method) %}{% endcall %}
    {%- call rb::async_handle_error_proc(method) %}{% endcall %}
    {%- call rb::async_throws_dispatch(method) %}{% endcall %}
  end

  {%- else %}
  # Callback method for {{ method.name() }}
  {{ method.name()|enum_name_rb }}_CALLBACK = Proc.new do |uniffi_handle, {%- for arg in method.arguments() %} {{ arg.name()|var_name_rb }},{% endfor %} uniffi_out_return, uniffi_call_status|
    uniffi_obj = {{ self::canonical_name(cbi.as_type().borrow()) }}FfiConverter.lift uniffi_handle
    {%- call rb::make_call_proc(method) %}{% endcall %}
    {%- call rb::write_return_value_proc(method) %}{% endcall %}
    {%- call rb::sync_throws_dispatch(method) %}{% endcall %}
  end
  {%- endif %}
  {%- endfor %}

  # Free callback: removes the handle from the map.
  UNIFFI_FREE_CALLBACK = Proc.new do |uniffi_handle|
    {{ self::canonical_name(cbi.as_type().borrow()) }}FfiConverter.handle_map.remove uniffi_handle
  end

  # Clone callback: clones the handle in the map.
  UNIFFI_CLONE_CALLBACK = Proc.new do |uniffi_handle|
    {{ self::canonical_name(cbi.as_type().borrow()) }}FfiConverter.handle_map.clone_handle uniffi_handle
  end

  # Create the VTable struct instance.
  UNIFFI_VTABLE = UniFFILib::{{ vtable_def.name() }}.new
  UNIFFI_VTABLE[:uniffi_free] = UNIFFI_FREE_CALLBACK
  UNIFFI_VTABLE[:uniffi_clone] = UNIFFI_CLONE_CALLBACK
  {%- for (ffi_callback, method)  in vtable_methods.iter() %}
  UNIFFI_VTABLE[:{{ method.name() }}] = {{ method.name()|enum_name_rb }}_CALLBACK
  {%- endfor %}

  # Register the VTable with Rust.
  UniFFILib.{{ ffi_init_callback.name() }}(UNIFFI_VTABLE)
end
