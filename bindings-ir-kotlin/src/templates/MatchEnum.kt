{%- set enum = get_enum(name=name) -%}
when ({{ value }}) {
    {%- for arm in arms %}
    {%- if arm.ir_type == "Variant" %}
    {%- set variant_def = enum|variant(name=arm.variant) -%}
    {%- if enum|has_fields %}
    is {{ name }}.{{ arm.variant }} -> {
        // Bind variables for the match
        {%- for item in zip(field=variant_def.fields, var=arm.vars) %}
        val {{ item.var }} = {{ value }}.{{ item.field.name }}
        {%- endfor %}
        {{ arm.block }}
    }
    {%- else %}
    {{ name }}.{{ arm.variant }} -> {
        {{ arm.block }}
    }
    {%- endif %}
    {%- elif arm.ir_type == "Default" %}
    else -> {
        {{ arm.block }}
    }
    {%- endif %}
    {%- endfor %}
}
