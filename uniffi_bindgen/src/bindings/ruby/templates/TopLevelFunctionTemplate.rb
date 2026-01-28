{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def self.{{ func.name()|fn_name_rb }}({%- call rb::arg_list_decl(func) %}{% endcall -%})
  {%- call rb::setup_args(func) %}{% endcall %}
  result = {% call rb::to_ffi_call(func) %}{% endcall %}
  return {{ "result"|lift_rb(return_type) }}
end

{% when None %}

def self.{{ func.name()|fn_name_rb }}({%- call rb::arg_list_decl(func) %}{% endcall -%})
  {%- call rb::setup_args(func) %}{% endcall %}
  {% call rb::to_ffi_call(func) %}{% endcall %}
end
{% endmatch %}
