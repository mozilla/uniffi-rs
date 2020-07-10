{%- match func.return_type() -%}
{%- when Some with (return_type) %}

    fun {{ func.name()|fn_name_kt }}(
        {%- for arg in func.arguments() %}
            {{ arg.name() }}: {{ arg.type_()|type_kt }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    ): {{ return_type|type_kt }} {
        val _retval = _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
            {%- for arg in func.arguments() %}
            {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        return {{ "_retval"|lift_kt(return_type) }}
    }

{% when None -%}

    fun {{ func.name()|fn_name_kt }}(
        {%- for arg in func.arguments() %}
            {{ arg.name() }}: {{ arg.type_()|type_kt }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    ) {
        UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
            {%- for arg in func.arguments() %}
            {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    }

{%- endmatch %}