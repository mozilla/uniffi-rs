{%- set enum = get_enum(name=name) -%}
{{ name }}.{{ variant }}{%- if enum|has_fields%}({{ values|comma_join }}){%- endif %}
