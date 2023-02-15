
{% match struct.documentation() -%}
  {% when Some with (docs) %}
/**
{% for line in docs.description.lines() %} * {{ line }} 
{% endfor %} */
  {%- when None %}
{%- endmatch %}