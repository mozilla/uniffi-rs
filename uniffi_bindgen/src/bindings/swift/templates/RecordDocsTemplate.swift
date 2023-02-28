{% match struct.documentation() -%}
  {% when Some with (docs) %}
/**
{% for line in docs.description.lines() %} * {{ line }} 
{% endfor %}
{%- if struct.has_fields_documentation() %} *
 * - Parameters:
{% endif -%}
{% for f in struct.fields() -%}
{% match f.documentation() -%}
{% when Some with (docs) %} *   - {{ f.name() }}: {{ docs }}
{% when None %}
{%- endmatch %}
{%- endfor %} */
  {%- when None %}
{%- endmatch %}