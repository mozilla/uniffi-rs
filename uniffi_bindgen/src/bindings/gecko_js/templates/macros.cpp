{# /* Calls an FFI function. */ #}
{%- macro to_ffi_call(context, func) -%}
  {%- call to_ffi_call_head(context, func, "err", "loweredRetVal_") -%}
  {%- call _to_ffi_call_tail(context, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function with an initial argument. */ #}
{%- macro to_ffi_call_with_prefix(context, prefix, func) %}
  {{ context.ffi_rusterror_type() }} err = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi(context) }} loweredRetVal_ ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {{ prefix }}
    {%- let args = func.webidl_arguments() -%}
    {%- if !args.is_empty() %},{% endif -%}
    {%- for arg in args %}
    {{ arg.type_()|lower_cpp(arg.name(), context) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    , &err
  );
  {%- call _to_ffi_call_tail(context, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function without handling errors or lifting the return
      value. Used in the implementation of `to_ffi_call`, and for
      constructors. */ #}
{%- macro to_ffi_call_head(context, func, error, result) %}
  {{ context.ffi_rusterror_type() }} {{ error }} = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi(context) }} {{ result }} ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {%- let args = func.webidl_arguments() -%}
    {%- for arg in args %}
    {{ arg.type_()|lower_cpp(arg.name(), context) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    {% if !args.is_empty() %}, {% endif %}&{{ error }}
  );
{%- endmacro -%}

{# /* Handles errors and lifts the return value from an FFI function. */ #}
{%- macro _to_ffi_call_tail(context, func, err, result) %}
  if ({{ err }}.mCode) {
    {%- match func.cpp_throw_by() %}
    {%- when ThrowBy::ErrorResult with (rv) %}
    {# /* TODO: Improve error throwing. See https://github.com/mozilla/uniffi-rs/issues/295
          for details. */ -#}
    {{ rv }}.ThrowOperationError({{ err }}.mMessage);
    {%- when ThrowBy::Assert %}
    MOZ_ASSERT(false);
    {%- endmatch %}
    return {% match func.cpp_return_type() %}{% when Some with (type_) %}{{ type_|dummy_ret_value_cpp(context) }}{% else %}{% endmatch %};
  }
  {%- match func.cpp_return_by() %}
  {%- when ReturnBy::OutParam with (name, type_) %}
  DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, name, context) }};
  MOZ_RELEASE_ASSERT(ok_);
  {%- when ReturnBy::Value with (type_) %}
  {{ type_|type_cpp(context) }} retVal_;
  DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, "retVal_", context) }};
  MOZ_RELEASE_ASSERT(ok_);
  return retVal_;
  {%- when ReturnBy::Void %}{%- endmatch %}
{%- endmacro -%}
