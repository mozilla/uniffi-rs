{# /* Calls an FFI function. */ #}
{%- macro to_ffi_call(namespace, func) -%}
  {%- call to_ffi_call_head(namespace, func, "err", "loweredRetVal_") -%}
  {%- call _to_ffi_call_tail(namespace, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function with an initial argument. */ #}
{%- macro to_ffi_call_with_prefix(namespace, prefix, func) %}
  RustError err = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi }} loweredRetVal_ ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {{ prefix }}
    {%- let args = func.arguments() -%}
    {%- if !args.is_empty() %},{% endif -%}
    {%- for arg in args %}
    {{ namespace|lower_cpp(arg.type_(), arg.name()) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    , &err
  );
  {%- call _to_ffi_call_tail(namespace, func, "err", "loweredRetVal_") -%}
{%- endmacro -%}

{# /* Calls an FFI function without handling errors or lifting the return
      value. Used in the implementation of `to_ffi_call`, and for
      constructors. */ #}
{%- macro to_ffi_call_head(namespace, func, error, result) %}
  RustError {{ error }} = {0, nullptr};
  {% match func.ffi_func().return_type() %}{% when Some with (type_) %}const {{ type_|type_ffi }} {{ result }} ={% else %}{% endmatch %}{{ func.ffi_func().name() }}(
    {%- let args = func.arguments() -%}
    {%- for arg in args %}
    {{ namespace|lower_cpp(arg.type_(), arg.name()) }}{%- if !loop.last %},{% endif -%}
    {%- endfor %}
    {% if !args.is_empty() %}, {% endif %}&{{ error }}
  );
{%- endmacro -%}

{# /* Handles errors and lifts the return value from an FFI function. */ #}
{%- macro _to_ffi_call_tail(namespace, func, err, result) %}
  if ({{ err }}.mCode) {
    {%- match func.throw_by() %}
    {%- when ThrowBy::ErrorResult with (rv) %}
    {# /* TODO: Improve error throwing. See https://github.com/mozilla/uniffi-rs/issues/295
          for details. */ -#}
    {{ rv }}.ThrowOperationError({{ err }}.mMessage);
    {%- when ThrowBy::Assert %}
    MOZ_ASSERT(false);
    {%- endmatch %}
    return {% match func.binding_return_type() %}{% when Some with (type_) %}{{ type_|dummy_ret_value_cpp }}{% else %}{% endmatch %};
  }
  {%- match func.return_by() %}
  {%- when ReturnBy::OutParam with (name, type_) %}
  DebugOnly<bool> ok_ = {{ namespace|lift_cpp(type_, result, name) }};
  MOZ_RELEASE_ASSERT(ok_);
  {%- when ReturnBy::Value with (type_) %}
  {{ type_|type_cpp }} retVal_;
  DebugOnly<bool> ok_ = {{ namespace|lift_cpp(type_, result, "retVal_") }};
  MOZ_RELEASE_ASSERT(ok_);
  return retVal_;
  {%- when ReturnBy::Void %}{%- endmatch %}
{%- endmacro -%}
