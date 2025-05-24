{%- let callable = func.callable %}
{% if callable.is_async %}async {% endif %}def {{ callable.name }}({% include "CallableArgs.py" %}) -> {{ callable.return_type.type_name }}:
    {{ func.docstring|docstring(4) -}}
    {%- filter indent(4) %}
    {%- include "CallableBody.py" %}
    {%- endfilter %}
