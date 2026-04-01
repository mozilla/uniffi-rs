{%- let type_name = rec.name_kt() %}
{%- if !rec.fields.is_empty() %}
data class {{ type_name }} (
    {%- for field in rec.fields.iter() %}
    {% if rec.immutable %}val {% else %}var {% endif %}
    {{- field.name_kt() }}: {{ field.ty.type_kt -}},
    {%- endfor %}
) {
    companion object
}
{%- else -%}
class {{ type_name }} {
    override fun equals(other: Any?): kotlin.Boolean {
        return other is {{ type_name }}
    }

    override fun hashCode(): kotlin.Int {
        return javaClass.hashCode()
    }

    companion object
}
{%- endif %}
