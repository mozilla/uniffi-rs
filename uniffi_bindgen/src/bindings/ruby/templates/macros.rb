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

{#- Helper: emit the opening of a rust_call or rust_call_with_error call.
    The reader is a Method object from error_reader_method_expr.
-#}
{%- macro rust_call_head(func) -%}
    {%- match self.error_reader_method_expr(func) %}
    {%- when Some with (reader) %}
    ::{{ self.module_name() }}.rust_call_with_error({{ reader }},
    {%- when None %}
    ::{{ self.module_name() }}.rust_call(
    {%- endmatch -%}
{%- endmacro -%}

{#- Emit the error-reader argument for async FFI calls: Method object or Ruby `nil`. -#}
{%- macro error_reader_expr(func) -%}
    {%- match self.error_reader_method_expr(func) %}
    {%- when Some with (reader) %}{{ reader }}
    {%- when None %}nil
    {%- endmatch %}
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
    {{ func|lower_method_self_rb(self) }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- self.lower_rb(arg.name()|var_name_rb, arg.as_type().borrow())? }}
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
        {%- when Some(_) %} = {{ self.arg_default_rb(arg)? }}
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

{%- macro to_ffi_call_async(func, prefix = "") -%}
    ::{{ self.module_name() }}.uniffi_rust_call_async(
      UniFFILib.{{ func.ffi_func().name() }}(
        {%- if !prefix.is_empty() %}{{- prefix }},{% endif %}
        {%- call _arg_list_ffi_call(func) %}{% endcall -%}
      ),
      :{{ func.ffi_rust_future_poll(ci) }},
      :{{ func.ffi_rust_future_cancel(ci) }},
      :{{ func.ffi_rust_future_complete(ci) }},
      :{{ func.ffi_rust_future_free(ci) }},
      {%- match func.return_type() %}
      {%- when Some with (return_type) %}
      Proc.new { |v| {{ self.lift_rb("v", return_type)? }} },
      {%- when None %}
      Proc.new { |v| nil },
      {%- endmatch %}
      {%- call error_reader_expr(func) %}{% endcall %}
    )
{%- endmacro %}

{%- macro to_ffi_call_async_constructor(func) %}
    ::{{ self.module_name() }}.uniffi_rust_call_async(
      UniFFILib.{{ func.ffi_func().name() }}(
        {%- call _arg_list_ffi_call(func) %}{% endcall -%}
      ),
      :{{ func.ffi_rust_future_poll(ci) }},
      :{{ func.ffi_rust_future_cancel(ci) }},
      :{{ func.ffi_rust_future_complete(ci) }},
      :{{ func.ffi_rust_future_free(ci) }},
      Proc.new { |v| v },
      {%- call error_reader_expr(func) %}{% endcall %}
    )
{%- endmacro %}

{#- Thin wrapper: delegates to to_ffi_call_async with an explicit prefix. -#}
{%- macro to_ffi_call_with_prefix_async(prefix, func) -%}
    {%- call to_ffi_call_async(func, prefix) %}{% endcall %}
{%- endmacro %}

{%- macro setup_args(func) %}
    {%- for arg in func.arguments() %}
    {{ arg.name()|var_name_rb }} = {{ self.coerce_rb(arg.name()|var_name_rb, arg.as_type().borrow())? }}
    {{ self.check_lower_rb(arg.name()|var_name_rb, arg.as_type().borrow())? }}
    {% endfor -%}
{%- endmacro -%}

{%- macro setup_args_extra_indent(meth) %}
        {%- for arg in meth.arguments() %}
        {{ arg.name()|var_name_rb }} = {{ self.coerce_rb(arg.name()|var_name_rb, arg.as_type().borrow())? }}
        {{ self.check_lower_rb(arg.name()|var_name_rb, arg.as_type().borrow())? }}
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
        {{ self.lift_rb(arg.name(), arg.as_type().borrow())? }}{% if !loop.last %},{% endif %}
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
      result_struct[:return_value] = {{ self.lower_rb("return_value", return_type)? }}
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
      lowered = {{ self.lower_rb("v", return_type)? }}
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

{#- Emit the error-specific trailing arguments (error class, lower-proc)
    for a sync/async trait-interface call.
    `lower_type` is the Type to use in the lower expression
    (error_type itself for direct errors, builtin for Custom-wrapped errors).
-#}
{%- macro trait_call_error_args(name, module_path, lower_type) %}
      {%- if self.is_external_module(module_path) %}
      ::{{ self.external_type_module(module_path) }}::{{ name|class_name_rb }},
      {%- else %}
      {{ name|class_name_rb }},
      {%- endif %}
      Proc.new { |e| {{ self.lower_rb("e", lower_type)? }} }
{%- endmacro %}

{#-
// Dispatch the throws type for a sync callback/trait-interface method.
// Caller must have in scope: uniffi_call_status, make_call, write_return_value.
-#}
{%- macro sync_throws_dispatch(method) %}
    {%- match method.throws_type() %}
    {%- when None %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
    )
    {%- when Some with (error_type) %}
    {%- match error_type %}
    {%- when Type::Enum { name, module_path, .. } | Type::Object { name, module_path, .. } %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
      {%- call trait_call_error_args(name, module_path, error_type) %}{% endcall %}
    )
    {%- when Type::Custom { builtin, .. } %}
    {%- match builtin.borrow() %}
    {%- when Type::Enum { name, module_path, .. } | Type::Object { name, module_path, .. } %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call(
      uniffi_call_status,
      make_call,
      write_return_value,
      {%- call trait_call_error_args(name, module_path, builtin) %}{% endcall %}
    )
    {%- else %}
    raise RuntimeError, "Unsupported custom error type"
    {%- endmatch %}
    {%- else %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call(
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
    ::{{ self.module_name() }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- when Some with (error_type) %}
    {%- match error_type %}
    {%- when Type::Enum { name, module_path, .. } | Type::Object { name, module_path, .. } %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
      {%- call trait_call_error_args(name, module_path, error_type) %}{% endcall %}
    )
    {%- when Type::Custom { builtin, .. } %}
    {%- match builtin.borrow() %}
    {%- when Type::Enum { name, module_path, .. } | Type::Object { name, module_path, .. } %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
      {%- call trait_call_error_args(name, module_path, builtin) %}{% endcall %}
    )
    {%- else %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- endmatch %}
    {%- else %}
    ::{{ self.module_name() }}.uniffi_trait_interface_call_async(
      make_call,
      uniffi_out_dropped_callback,
      handle_success,
      handle_error,
    )
    {%- endmatch %}
    {%- endmatch %}
{%- endmacro %}
