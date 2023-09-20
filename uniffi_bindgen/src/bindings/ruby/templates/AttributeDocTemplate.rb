{% match field.documentation() -%}
{% when Some with (docs) %}  # @return [{{ canonical_name(field.type_()) }}] {{ docs }}
{% when None %}
{%- endmatch %}

