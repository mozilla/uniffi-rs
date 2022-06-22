{%- set class = get_class(name=name) -%}
{%- if class.destructor -%}
{{ name }}({{ values|comma_join }}).also { objectCleaner.register(it, {{ name }}.Cleaner({% for f in class.fields %}it.{{ f.name }}, {% endfor %}it.shouldRunDestructor)) }
{%- else %}
{{ name }}({{ values|comma_join }})
{%- endif %}
