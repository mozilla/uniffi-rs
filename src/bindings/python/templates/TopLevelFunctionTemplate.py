def {{ func.name()|fn_name_py }}({% for arg in func.arguments() %}{{ arg.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
    {%- for arg in func.arguments() %}
    {{ arg.name()|coerce_py(arg.type_()) }}
    {%- endfor %}
    _retval = _UniFFILib.{{ func.ffi_func().name() }}(
        {%- for arg in func.arguments() %}
        {{ arg.name()|lower_py(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    )
    return {% match func.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lift_py(return_type) }}{% else %}None{% endmatch %}