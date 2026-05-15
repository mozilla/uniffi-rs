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

  # Callback method for {{ method.name() }}
  {{ method.name()|enum_name_rb }}_CALLBACK = Proc.new do |uniffi_handle, {%- for arg in method.arguments() %} {{ arg.name()|var_name_rb }},{% endfor %} uniffi_out_return, uniffi_call_status|
    uniffi_obj = {{ self::canonical_name(cbi.as_type().borrow()) }}FfiConverter.lift uniffi_handle

    make_call = Proc.new do
      uniffi_obj.{{ method.name()|fn_name_rb }}(
        {%- for arg in method.arguments() %}
        {{ arg.name()|lift_rb(arg.as_type().borrow(), config) }}{% if !loop.last %},{% endif %}
        {%- endfor %}
      )
    end

    {%- match method.return_type() %}
    {%- when Some with (return_type) %}
    write_return_value = Proc.new do |v|
      lowered = {{ "v"|lower_rb(return_type, config) }}
      {%- let ffi_type_name = return_type|ffi_write_return_rb %}
      {%- if ffi_type_name == "rustbuffer" %}
      # Write a RustBuffer struct into the out pointer
      out_buf = RustBuffer.new uniffi_out_return
      out_buf[:capacity] = lowered[:capacity]
      out_buf[:len] = lowered[:len]
      out_buf[:data] = lowered[:data]
      {%- else %}
      uniffi_out_return.{{ ffi_type_name }}(lowered)
      {%- endif %}
    end
    {%- when None %}
    # No return value, so do nothing.
    write_return_value = Proc.new { |_v| }
    {%- endmatch %}

    {%- match method.throws_type() %}
    {%- when None %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value
    )
    {%- when Some with (error_type) %}
    {%- match error_type %}
    {%- when Type::Enum { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_with_error(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(error_type, config) }} }
    )
    {%- when Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_with_error(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(error_type, config) }} }
    )
    {%- when Type::Custom { builtin, .. } %}
    {%- match builtin.borrow() %}
    {%- when Type::Enum { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_with_error(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(builtin, config) }} }
    )
    {%- when Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_with_error(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(builtin, config) }} }
    )
    {%- else %}
    raise RuntimeError, "Unsupported custom error type for callback interface"
    {%- endmatch %}
    {%- else %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value
    )
    {%- endmatch %}
    {%- endmatch %}
  end
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
