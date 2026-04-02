{%- let type_name = en.name_kt() %}

{#
// Kotlin's `enum class` construct doesn't support variants with associated data,
// but is a little nicer for consumers than its `sealed class` enum pattern.
// So, we switch here, using `enum class` for enums with no associated data
// and `sealed class` for the general case.
#}

{%- if en.is_flat %}

{% if !en.discr_specified %}
enum class {{ type_name }} {
    {% for v in en.variants -%}
    {{ v.name_kt }}{% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object
}
{%- else %}
enum class {{ type_name }}(val value: {{ en.discr_type.type_kt }}) {
    {% for v in en.variants -%}
    {{ v.name_kt }}({{ v.discr.lit_kt }}){% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object
}
{% endif %}
{% else %}
sealed class {{ type_name }}
{
    {% for v in en.variants -%}
    {% if v.fields.is_empty() -%}
    object {{ v.name_kt }} : {{ type_name }}()
    {% else -%}
    data class {{ v.name_kt }}(
        {%- for f in v.fields -%}
        val {{ f.name_kt() }}: {{ f.ty.type_kt }}
        {%- if let Some(default) = f.default %} = {{ default.default_kt }}{% endif %},
        {%- endfor -%}
    ) : {{ type_name }}()
    {%- endif %}
    {% endfor %}

    companion object
}
{% endif %}
