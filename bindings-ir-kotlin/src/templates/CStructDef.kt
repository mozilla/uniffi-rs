@com.sun.jna.Structure.FieldOrder({% for field in fields %}"{{ field.name }}"{% if not loop.last %}, {% endif %}{% endfor %})
open class {{ name }} (
    {# Note: jna.Structure requires that all fields are `var` #}
    {%- for field in fields %}
    @JvmField var {{ field.name }}: {{ field.type }} = {{ field.type|ffi_default }},
    {%- endfor %}
) : com.sun.jna.Structure(), com.sun.jna.Structure.ByValue { }

{%- for field in fields %}
operator fun {{ name }}.component{{ loop.index }}() = this.{{ field.name }}
{%- endfor %}
