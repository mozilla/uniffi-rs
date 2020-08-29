{# /* Calls an FFI function. */ #}
{%- macro to_ffi_call(func) -%}
  {%- call to_ffi_call_head(func, "err", "loweredRetVal_") -%}
  {%- call _to_ffi_call_tail(func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function with an initial argument. */ #}
{%- macro to_ffi_call_with_prefix(prefix, func) %}
  RustError err = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi }} loweredRetVal_ ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {{ prefix }}
    {%- let args = func.arguments() -%}
    {%- if !args.is_empty() %},{% endif -%}
    {%- for arg in args %}
    {{ arg.type_()|lower_cpp(arg.name()) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    {%- if func.ffi_func().has_out_err() -%}
    , &err
    {%- endif %}
  );
  {%- call _to_ffi_call_tail(func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function without handling errors or lifting the return
      value. Used in the implementation of `to_ffi_call`, and for
      constructors. */ #}
{%- macro to_ffi_call_head(func, error, result) %}
  RustError {{ error }} = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi }} {{ result }} ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {%- let args = func.arguments() -%}
    {%- for arg in args %}
    {{ arg.type_()|lower_cpp(arg.name()) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    {%- if func.ffi_func().has_out_err() -%}
    {% if !args.is_empty() %}, {% endif %}&{{ error }}
    {%- endif %}
  );
{%- endmacro -%}

{# /* Handles errors and lifts the return value from an FFI function. */ #}
{%- macro _to_ffi_call_tail(func, err, result) %}
  if ({{ err }}.mCode) {
    {%- match func.binding_throw_by() %}
    {%- when ThrowBy::ErrorResult with (rv) %}
    {{ rv }}.ThrowOperationError({{ err }}.mMessage);
    {%- when ThrowBy::Assert %}
    MOZ_ASSERT(false);
    {%- endmatch %}
    return {% match func.binding_return_type() %}{% when Some with (type_) %}{{ type_|dummy_ret_value_cpp }}{% else %}{% endmatch %};
  }
  {%- match func.binding_return_by() %}
  {%- when ReturnBy::OutParam with (name, type_) %}
  DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, name) }};
  MOZ_ASSERT(ok_);
  {%- when ReturnBy::Value with (type_) %}
  {{ type_|type_cpp }} retVal_;
  DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, "retVal_") }};
  MOZ_ASSERT(ok_);
  return retVal_;
  {%- when ReturnBy::Void %}{%- endmatch %}
{%- endmacro -%}
