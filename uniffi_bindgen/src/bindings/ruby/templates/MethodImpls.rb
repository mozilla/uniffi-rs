{# Shared template for generating instance methods on Records and non-flat Enums.
# The caller must bind `methods` in the scope before including this file, e.g.:
#   {% let methods = rec.methods() %}
#   {% include "MethodImpls.rb" %}
# Async methods are skipped (not implemented in Ruby yet).
#}
{% for meth in methods -%}
{%- if meth.is_async() %}{% continue %}{%- endif %}
{%- match meth.return_type() -%}

{%- when Some with (return_type) -%}
def {{ meth.name()|fn_name_rb }}{% call rb::arg_list_decl(meth) %}{% endcall %}
  {%- call rb::setup_args_extra_indent(meth) %}{% endcall %}
  result = {% call rb::to_ffi_call_with_lower_self(meth) %}{% endcall %}
  return {{ "result"|lift_rb(return_type, config) }}
end

{%- when None %}
def {{ meth.name()|fn_name_rb }}{% call rb::arg_list_decl(meth) %}{% endcall %}
  {%- call rb::setup_args_extra_indent(meth) %}{% endcall %}
  result = {% call rb::to_ffi_call_with_lower_self(meth) %}{% endcall %}
end

{%- endmatch %}
{% endfor %}
