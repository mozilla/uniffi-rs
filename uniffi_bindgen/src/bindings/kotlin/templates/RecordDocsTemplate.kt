
{% match rec.documentation() -%}
  {% when Some with (docs) %}
/**
* {{ docs.description }}
*/
  {%- when None %}
{%- endmatch %}