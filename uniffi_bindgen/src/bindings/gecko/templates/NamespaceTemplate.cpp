#include "mozilla/dom/{{ ci.namespace()|class_name_webidl }}.h"

namespace mozilla {
namespace dom {

{% for func in functions %}
{#- /* Return type. `void` for methods that return nothing, or return their
       value via an out param. */ #}
{%- match ReturnPosition::for_function(func) -%}
{%- when ReturnPosition::OutParam with (_) -%}
void
{%- when ReturnPosition::Void %}
void
{%- when ReturnPosition::Return with (type_) %}
{{ type_|ret_type_cpp }}
{%- endmatch %}
{{ ci.namespace()|class_name_cpp }}::{{ func.name()|fn_name_cpp }}(
    GlobalObject& aGlobal
    {%- let args = func.arguments() %}
    {%- if !args.is_empty() %}, {% endif %}
    {%- for arg in args %}
    {{ arg.type_()|arg_type_cpp }} {{ arg.name() }}{%- if !loop.last %}, {% endif %}
    {%- endfor -%}
    {#- /* Out param returns. */ #}
    {%- match ReturnPosition::for_function(func) -%}
    {%- when ReturnPosition::OutParam with (type_) -%}
    , {{ type_|ret_type_cpp }} aLiftedRetVal
    {% else %}{% endmatch %}
    {#- /* Errors. */ #}
    {%- if func.throws().is_some() %}
    {%- if ReturnPosition::for_function(func).is_out_param() || !args.is_empty() %}, {% endif %}
    ErrorResult& aRv
    {%- endif %}
) {
  {%- if func.throws().is_some() %}
  RustError err{0, nullptr};
  {% endif %}
  {%- if func.return_type().is_some() %}auto loweredRetVal = {% endif %}{{ func.ffi_func().name() }}(
    {%- for arg in func.arguments() %}
      {{- arg.name()|lower_cpp(arg.type_()) }}
      {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
    {%- if func.throws().is_some() %}
      {%- if !args.is_empty() %},{% endif %}&err
    {% endif %}
  );
  {%- if func.throws().is_some() %}
  if (err.mCode) {
    aRv.ThrowOperationError(err.mMessage);
    {% match self::ret_default_value_cpp(func) -%}
    {%- when Some with (val) -%}
    return {{ val }};
    {% else %}
    return;{%- endmatch %}
  }
  {%- endif %}
  {% match ReturnPosition::for_function(func) -%}
  {%- when ReturnPosition::OutParam with (type_) -%}
  {{ "loweredRetVal"|lift_cpp("aLiftedRetVal", type_) }};
  {%- when ReturnPosition::Return with (type_) %}
  {{ type_|type_cpp }} result;
  {{ "loweredRetVal"|lift_cpp("result", type_) }};
  return result;
  {%- when ReturnPosition::Void %}{%- endmatch %}
}
{% endfor %}

}  // namespace dom
}  // namespace mozilla
