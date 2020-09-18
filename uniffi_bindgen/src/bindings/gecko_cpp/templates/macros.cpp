{# /* Return type... */ #}
{%- macro decl_ret_type(func) -%}
  {%- match func.throws() -%}
    {%- when Some with (err) -%}
      {%- match func.return_type() -%}
        {%- when Some with (type_) -%}
          Result<{{ type_|type_cpp }}, {{ err }}>
        {%- when None -%}
          Result<Ok, {{ err }}>
      {%- endmatch -%}
    {%- when None -%}
      {%- match func.return_type() -%}
        {%- when Some with (type_) -%}
          {{ type_|type_cpp }}
        {%- when None -%}
          void
      {%- endmatch -%}
  {%- endmatch -%}
{%- endmacro -%}

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
    , &err
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
    {% if !args.is_empty() %}, {% endif %}&{{ error }}
  );
{%- endmacro -%}

{# /* Handles errors and lifts the return value from an FFI function. */ #}
{%- macro _to_ffi_call_tail(func, err, result) %}
  {%- match func.throws() %}
  {%- when Some with (name) %}
  auto maybeError = {{ name|class_name_cpp }}::FromConsuming(err);
  if (maybeError.isSome()) {
    return Err(maybeError.extract());
  }
  {% else %}
  if ({{ err }}.mCode) {
    MOZ_ASSERT(false);
    return {% match func.return_type() %}{% when Some with (type_) %}{{ type_|dummy_ret_value_cpp }}{% else %}{% endmatch %};
  }
  {% endmatch %}
  {%- match func.return_type() %}
  {%- when Some with (type_) %}
  {{ type_|type_cpp }} retVal_;
  DebugOnly<bool> ok_ = {{ type_|lift_cpp(result, "retVal_") }};
  MOZ_ASSERT(ok_);
  return std::move(retVal_);
  {% else %}{%- endmatch %}
{%- endmacro -%}
