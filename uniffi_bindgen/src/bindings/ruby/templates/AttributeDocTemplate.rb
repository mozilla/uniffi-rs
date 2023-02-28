{% match field.documentation() -%}
{% when Some with (docs) %}  # @return [{{ field.type_()|type_name }}] {{ docs }}
{% when None %}
{%- endmatch %}

