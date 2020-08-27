{# /* Functions that throw an error via `aRv` and return results by value still
      need to return dummy values, even though they're discarded by the caller.
      This macro returns a suitable "empty value". */ #}
{%- macro bail(func) -%}
{%- match func.return_type().and_then(self::default_return_value_cpp) -%}
{%- when Some with (value) -%}
return {{ value }};
{%- else -%}
return;
{%- endmatch -%}
{%- endmacro -%}

{# /* Declares a return type for a function or method. Functions can return
      values directly or via "out params"; this macro handles both cases. */ #}
{%- macro decl_return_type(func) -%}
  {%- match func.return_type() -%}
    {%- when Some with (type_) -%}
      {%- match ReturnPosition::for_type(type_) -%}
        {%- when ReturnPosition::Return with (type_) -%} {{ type_|ret_type_cpp }}
        {%- when ReturnPosition::OutParam with (_) -%} void
      {%- endmatch -%}
    {%- else -%} void
  {%- endmatch -%}
{%- endmacro -%}

{# /* Declares a list of arguments for a WebIDL constructor. A constructor takes
      a `GlobalObject&` as its first argument, followed by its declared
      arguments, and then an optional `ErrorResult` if it throws. */ #}
{%- macro decl_constructor_args(cons) -%}
  GlobalObject& aGlobal
  {%- let args = cons.arguments() -%}
  {%- if !args.is_empty() -%}, {%- endif -%}
  {%- for arg in args -%}
  {{ arg.type_()|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %}, {%- endif -%}
  {%- endfor -%}
  {%- if cons.throws().is_some() -%}
  , ErrorResult& aRv
  {%- endif -%}
{%- endmacro -%}

{# /* Declares a list of arguments for a WebIDL static method. A static or
      namespace method takes a `GlobalObject&` as its first argument, followed
      by its declared arguments, an optional "out param" for the return value,
      and an optional `ErrorResult` if it throws. */ #}
{%- macro decl_static_method_args(cons) -%}
  GlobalObject& aGlobal
  {%- let args = func.arguments() %}
  {%- if !args.is_empty() -%},{%- endif %}
  {%- for arg in args %}
  {{ arg.type_()|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %},{% endif %}
  {%- endfor -%}
  {%- call _decl_out_param(func) -%}
  {%- if cons.throws().is_some() %}
  , ErrorResult& aRv
  {%- endif %}
{%- endmacro -%}

{# /* Declares a list of arguments for a WebIDL interface method. An interface
      method takes its declared arguments, an optional "out param" for the
      return value, and an `ErrorResult&`. */ #}
{%- macro decl_method_args(func) -%}
  {%- let args = func.arguments() -%}
  {%- for arg in args -%}
  {{ arg.type_()|arg_type_cpp }} {{ arg.name() -}}{%- if !loop.last -%},{%- endif -%}
  {%- endfor -%}
  {%- call _decl_out_param(func) -%}
  {%- match func.return_type() -%}
    {%- when Some with (type_) -%}
      {%- match ReturnPosition::for_type(type_) -%}
        {%- when ReturnPosition::OutParam with (type_) -%},
        {%- else -%}
          {%- if !args.is_empty() %}, {% endif -%}
      {%- endmatch -%}
    {%- else -%}
      {%- if !args.is_empty() %}, {% endif -%}
  {%- endmatch -%}
  ErrorResult& aRv
{%- endmacro -%}

{# /* Returns a result from a function or method. This lifts the result from the
      given `var`, and returns it by value or via an "out param" depending on
      the function's return type. */ #}
{%- macro return(func, var) -%}
{% match func.return_type() -%}
{%- when Some with (type_) -%}
  {% match ReturnPosition::for_type(type_) -%}
  {%- when ReturnPosition::OutParam with (type_) -%}
  DebugOnly<bool> ok_ = detail::ViaFfi<{{ type_|type_cpp }}, {{ type_|ret_type_ffi }}>::Lift({{ var }}, aRetVal_);
  MOZ_ASSERT(ok_);
  {%- when ReturnPosition::Return with (type_) %}
  {{ type_|type_cpp }} retVal_;
  DebugOnly<bool> ok_ = detail::ViaFfi<{{ type_|type_cpp }}, {{ type_|ret_type_ffi }}>::Lift({{ var }}, retVal_);
  MOZ_ASSERT(ok_);
  return retVal_;
  {%- endmatch %}
{% else -%}
  return;
{%- endmatch %}
{%- endmacro -%}

{# /* Lowers a list of function arguments for an FFI call. */ #}
{%- macro to_ffi_args(args) %}
  {%- for arg in args %}
    detail::ViaFfi<{{ arg.type_()|type_cpp }}, {{ arg.type_()|ret_type_ffi }}>::Lower({{ arg.name() }}){%- if !loop.last %}, {% endif -%}
  {%- endfor %}
{%- endmacro -%}

{# /* Declares an "out param" in the argument list. */ #}
{%- macro _decl_out_param(func) -%}
{%- match func.return_type() -%}
  {%- when Some with (type_) -%}
    {%- match ReturnPosition::for_type(type_) -%}
      {%- when ReturnPosition::OutParam with (type_) -%}
        {%- if !func.arguments().is_empty() -%},{%- endif -%}
        {{ type_|ret_type_cpp }} aRetVal_
      {%- else -%}
    {%- endmatch -%}
  {%- else -%}
{%- endmatch -%}
{%- endmacro -%}
