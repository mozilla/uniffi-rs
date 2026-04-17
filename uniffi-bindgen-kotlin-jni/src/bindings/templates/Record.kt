{%- let type_name = rec.name_kt() %}
{%- if !rec.fields.is_empty() %}
data class {{ type_name }} (
    {%- for field in rec.fields %}
    {% if rec.immutable %}val {% else %}var {% endif %}
    {{- field.name_kt() }}: {{ field.ty.type_kt -}}
    {%- if let Some(default) = field.default %} = {{ default.default_kt }}{% endif -%}
    ,
    {%- endfor %}
){% if rec.uniffi_trait_methods.ord_cmp.is_some() %}: Comparable<{{ type_name }}> {% endif %} {

    {%- let uniffi_trait_methods = rec.uniffi_trait_methods %}
    {% filter indent(4) %}{% include "UniffiTraitMethods.kt" %}{% endfilter %}

    companion object
}
{%- else -%}
class {{ type_name }} {
    override fun equals(other: Any?): Boolean {
        return other is {{ type_name }}
    }

    override fun hashCode(): Int {
        return javaClass.hashCode()
    }

    companion object
}
{%- endif %}
