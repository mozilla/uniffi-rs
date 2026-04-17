{%- let type_name = en.name_kt() %}

{%- match en.kotlin_kind %}
{#
// Kotlin's `enum class` construct doesn't support variants with associated data,
// but is a little nicer for consumers than its `sealed class` enum pattern.
// So, we switch here, using `enum class` for enums with no associated data
// and `sealed class` for the general case.
#}
{%- when KotlinEnumKind::EnumClass { discr_type: None } %}
enum class {{ type_name }} {
    {% for v in en.variants -%}
    {{ v.name_kt }}{% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object
}

{%- when KotlinEnumKind::EnumClass { discr_type: Some(discr_type) } %}
enum class {{ type_name }}(val value: {{ en.discr_type.type_kt }}) {
    {% for v in en.variants -%}
    {{ v.name_kt }}({{ v.discr.lit_kt }}){% if loop.last %};{% else %},{% endif %}
    {%- endfor %}

    companion object
}

{%- when KotlinEnumKind::SealedClass %}
sealed class {{ type_name }}
{%- if !en.base_classes.is_empty() %} : {{ en.base_classes|join(", ") }}{% endif %} {
    {% for v in en.variants -%}
    {% if en.self_type.is_used_as_error -%}
    {# error types always have use `class` for their variants #}
    class {{ v.name_kt }}(
        {%- for f in v.fields -%}
        val {{ f.name_kt() }}: {{ f.ty.type_kt }}
        {%- if let Some(default) = f.default %} = {{ default.default_kt }}{% endif %},
        {%- endfor -%}
    ) : {{ type_name }}()
    {% elif v.fields.is_empty() -%}
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

    {%- let uniffi_trait_methods = en.uniffi_trait_methods %}
    {% filter indent(4) %}{% include "UniffiTraitMethods.kt" %}{% endfilter %}

    companion object
}

{%- when KotlinEnumKind::FlatError %}
sealed class {{ type_name }}(message: String) : kotlin.Exception(message) {
    {%- for v in en.variants %}
    class {{ v.name_kt }}(message: String) : {{ type_name }}(message)
    {%- endfor %}

    companion object
}

{%- endmatch %}
