{%- for arg in callable.arguments -%}
    {{ arg.name }}
    {%- match arg.default %}
    {%- when Some(default) %}: {% if !default.is_arg_literal() %}typing.Union[object, {{ arg.param_type_name() }}]{% else %}{{ arg.param_type_name() }}{% endif %} = {{ default.arg_literal }}
    {%- else %}: {{ arg.param_type_name() }}
    {%- endmatch %}
    {%- if !loop.last %},{% endif -%}
{%- endfor %}
