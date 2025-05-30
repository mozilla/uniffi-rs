{%- for arg in callable.arguments -%}
    {{ arg.name }}
    {%- match arg.default %}
    {%- when Some(literal) %}: typing.Union[object, {{ arg.ty.type_name }}] = _DEFAULT
    {%- else %}: {{ arg.ty.type_anno_name }}
    {%- endmatch %}
    {%- if !loop.last %},{% endif -%}
{%- endfor %}
