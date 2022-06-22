{%- if self|has_fields %}
// Kotlin's version of an enum with fields is set of sealed classses
{{ vis }} sealed interface {{ name }} {
    {%- for variant in variants %}
    data class {{ variant.name }}({{ variant.fields|comma_join }}): {{ name }}
    {%- endfor %}
}
{%- else %}
{{ vis }} enum class {{ name }} {
    {%- for variant in variants %}
    {{ variant.name }},
    {%- endfor %}
}
{%- endif %}
