{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_rb` in `lower_rb`)
#}

{#
// Returns the Ruby name for an enum variant field.
// For tuple-style fields (empty name is uniffi metadata) we generate a 
// positional name like v1, v2, ... using the 1-based loop index.
#}
{%- macro field_name(field, field_num) -%}
{%- if field.name().is_empty() -%}
values[{{- field_num - 1 -}}]
{%- else -%}
{{ field.name()|var_name_rb }}
{%- endif -%}
{%- endmacro -%}

{#- Helper: emit the opening of a rust_call or rust_call_with_error call. -#}
{%- macro rust_call_head(func) -%}
    {%- match func.throws_type() -%}
    {%- when Some(Type::Custom { builtin, .. }) -%}
      {%- match builtin.borrow() -%}
      {%- when Type::Enum { name, .. } | Type::Object { name, .. } -%}
      ::{{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
      {%- else -%}
      ::{{ ci.namespace()|class_name_rb }}.rust_call
      {%- endmatch -%}
    {%- when Some(Type::Enum { name, .. }) | Some(Type::Object { name, .. }) -%}
      ::{{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
    {%- else -%}
      ::{{ ci.namespace()|class_name_rb }}.rust_call(
    {%- endmatch -%}
{%- endmacro -%}
  
{%- macro to_ffi_call(func) -%}
    {%- call rust_call_head(func) %}{% endcall -%}
    :{{ func.ffi_func().name() }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
    {%- call rust_call_head(func) %}{% endcall -%}
    :{{ func.ffi_func().name() }},
    {{- prefix }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro to_ffi_call_with_lower_self(func) -%}
    {%- call rust_call_head(func) %}{% endcall -%}
    :{{ func.ffi_func().name() }},
    {{ func|lower_method_self_rb(config) }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|var_name_rb|lower_rb(arg.as_type().borrow(), config) }}
        {%- if !loop.last %},{% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Ruby declarations of methods, functions and constructors.
// Note the var_name_rb and type_rb filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_rb }}
        {%- match arg.default_value() %}
        {%- when Some(_) %} = {{ arg|arg_default_rb }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the UniFFILib function declarations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    [{%- for arg in func.arguments() -%}{{ arg.type_().borrow()|type_ffi }}, {% endfor -%}
    {%- if func.has_rust_call_status_arg() -%}RustCallStatus.by_ref{% endif -%}]
{%- endmacro -%}

{#- Helper: emit the error class name or nil for async rust calls. -#}
{%- macro throws_error_class_expr(func) %}
    {%- match func.throws_type() %}
    {%- when Some(Type::Custom { builtin, .. }) %}
      {%- match builtin.borrow() %}
      {%- when Type::Enum { name, .. } | Type::Object { name, .. } %}
    {{ name|class_name_rb }}
      {%- else %}
    nil
      {%- endmatch %}
    {%- when Some(Type::Enum { name, .. }) | Some(Type::Object { name, .. }) %}
    {{ name|class_name_rb }}
    {%- else %}
    nil
    {%- endmatch %}
{%- endmacro %}

{%- macro to_ffi_call_async(func, prefix = "") -%}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_rust_call_async(
      UniFFILib.{{ func.ffi_func().name() }}(
        {%- if !prefix.is_empty() %}{{- prefix }},{% endif %}
        {%- call _arg_list_ffi_call(func) %}{% endcall -%}
      ),
      :{{ func.ffi_rust_future_poll(ci) }},
      :{{ func.ffi_rust_future_complete(ci) }},
      :{{ func.ffi_rust_future_free(ci) }},
      {%- match func.return_type() %}
      {%- when Some with (return_type) %}
      Proc.new { |v| {{ "v"|lift_rb(return_type, config) }} },
      {%- when None %}
      Proc.new { |v| nil },
      {%- endmatch %}
      {%- call throws_error_class_expr(func) %}{% endcall %}
    )
{%- endmacro %}

{#- Async constructor variant: uses identity lift to return raw handle for uniffi_allocate -#}
{%- macro to_ffi_call_async_constructor(func) %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_rust_call_async(
      UniFFILib.{{ func.ffi_func().name() }}(
        {%- call _arg_list_ffi_call(func) %}{% endcall -%}
      ),
      :{{ func.ffi_rust_future_poll(ci) }},
      :{{ func.ffi_rust_future_complete(ci) }},
      :{{ func.ffi_rust_future_free(ci) }},
      Proc.new { |v| v },
      {%- call throws_error_class_expr(func) %}{% endcall %}
    )
{%- endmacro %}

{#- Thin wrapper: delegates to to_ffi_call_async with an explicit prefix. -#}
{%- macro to_ffi_call_with_prefix_async(prefix, func) -%}
    {%- call to_ffi_call_async(func, prefix) %}{% endcall %}
{%- endmacro %}

{%- macro setup_args(func) %}
    {%- for arg in func.arguments() %}
    {{ arg.name()|var_name_rb }} = {{ arg.name()|var_name_rb|coerce_rb(ci.namespace()|class_name_rb, arg.as_type().borrow(), config) }}
    {{ arg.name()|var_name_rb|check_lower_rb(arg.as_type().borrow(), config) }}
    {% endfor -%}
{%- endmacro -%}

{%- macro setup_args_extra_indent(meth) %}
        {%- for arg in meth.arguments() %}
        {{ arg.name()|var_name_rb }} = {{ arg.name()|var_name_rb|coerce_rb(ci.namespace()|class_name_rb, arg.as_type().borrow(), config) }}
        {{ arg.name()|var_name_rb|check_lower_rb(arg.as_type().borrow(), config) }}
        {%- endfor %}
{%- endmacro -%}

{#-
// Build the `make_call` Proc for callback/trait-interface methods (sync and async).
// Requires `uniffi_obj` to be in caller scope.
-#}
{%- macro make_call_proc(method) %}
    make_call = Proc.new do
      uniffi_obj.{{ method.name()|fn_name_rb }}(
        {%- for arg in method.arguments() %}
        {{ arg.name()|lift_rb(arg.as_type().borrow(), config) }}{% if !loop.last %},{% endif %}
        {%- endfor %}
      )
    end
{%- endmacro %}

{#-
// Build the `handle_success` Proc for async callback/trait-interface methods.
// Requires `uniffi_future_callback` and `uniffi_callback_data` in the caller scope.
-#}
{%- macro async_handle_success_proc(method) %}
    handle_success = Proc.new do |return_value|
      result_struct = UniFFILib::{{ method|foreign_future_result_rb }}.new
      {%- match method.return_type() %}
      {%- when Some with (return_type) %}
      result_struct[:return_value] = {{ "return_value"|lower_rb(return_type, config) }}
      result_struct[:call_status] = RustCallStatus.new
      {%- when None %}
      result_struct[:call_status] = RustCallStatus.new
      {%- endmatch %}
      uniffi_future_callback.call(uniffi_callback_data, result_struct)
    end
{%- endmacro %}

{#-
// Build the `handle_error` Proc for async callback/trait-interface methods.
// Requires `uniffi_future_callback` and `uniffi_callback_data` in the caller scope.
-#}
{%- macro async_handle_error_proc(method) %}
    handle_error = Proc.new do |status_code, error_buf|
      result_struct = UniFFILib::{{ method|foreign_future_result_rb }}.new
      {%- match method.return_type() %}
      {%- when Some with (return_type) %}
      result_struct[:return_value] = {{ return_type|ffi_default_value_rb }}
      {%- when None %}
      {%- endmatch %}

      error_status = RustCallStatus.new
      error_status[:code] = status_code
      error_status[:error_buf] = error_buf

      result_struct[:call_status] = error_status

      uniffi_future_callback.call(uniffi_callback_data, result_struct)
    end
{%- endmacro %}


{#-
// Build the `write_return_value` Proc for sync callback/trait-interface methods.
// Requires `uniffi_out_return` in the caller scope.
-#}
{%- macro write_return_value_proc(method) %}
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
    # No return value, so do nothing
    write_return_value = Proc.new { |_v| }
    {%- endmatch %}
{%- endmacro %}

{#-
// Dispatch the throws type for a sync callback/trait-interface method.
// Caller must have in scope: uniffi_call_status, make_call, write_return_value.
-#}
{%- macro sync_throws_dispatch(method) %}
    {%- match method.throws_type() %}
    {%- when None %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
    )
    {%- when Some with (error_type) %}
    {%- match error_type %}
    {%- when Type::Enum { name, .. } | Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(error_type, config) }} }
    )
    {%- when Type::Custom { builtin, .. } %}
    {%- match builtin.borrow() %}
    {%- when Type::Enum { name, .. } | Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(builtin, config) }} }
    )
    {%- else %}
    raise RuntimeError, "Unsupported custom error type"
    {%- endmatch %}
    {%- else %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value
    )
    {%- endmatch %}
    {%- endmatch %}
{%- endmacro %}

{#-
// Dispatch the throws type for an async callback/trait-interface method.
// Caller must have in scope: make_call, uniffi_out_dropped_callback, handle_success, handle_error.
-#}
{%- macro async_throws_dispatch(method) %}
    {%- match method.throws_type() %}
    {%- when None %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- when Some with (error_type) %}
    {%- match error_type %}
    {%- when Type::Enum { name, .. } | Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(error_type, config) }} }
    )
    {%- when Type::Custom { builtin, .. } %}
    {%- match builtin.borrow() %}
    {%- when Type::Enum { name, .. } | Type::Object { name, .. } %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
      {{ name|class_name_rb }},
      Proc.new { |e| {{ "e"|lower_rb(builtin, config) }} }
    )
    {%- else %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- endmatch %}
    {%- else %}
    ::{{ ci.namespace()|class_name_rb }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- endmatch %}
    {%- endmatch %}
{%- endmacro %}
