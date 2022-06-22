when ({{ value }}) {
    {%- for arm in arms %}
    {%- if arm.ir_type == "Value" %}
    {{ arm.value }} -> {
        {{ arm.block }}
    }
    {%- elif arm.ir_type == "Default" %}
    else -> {
        {{ arm.block }}
    }
    {%- endif %}
    {%- endfor %}
}
