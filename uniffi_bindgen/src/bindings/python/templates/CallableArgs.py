{%- for arg in callable.arguments -%}
    {{ arg.name }}
    {%- match arg.default %}
    {%- when Some(default) %}: {% if !default.is_arg_literal() %}typing.Union[object, {{ arg.ty.type_name }}]{% else %}{{ arg.ty.type_name }}{% endif %} = {{ default.arg_literal }}
    {%- else %}: {{ arg.ty.type_name }}
    {%- endmatch %}
    {%- if !loop.last %},{% endif -%}
{%- endfor %}
