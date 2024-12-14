{{ func|def }} {{ func.name }}({{ func|arg_list }}) -> {{ func|return_type }}:
    {{ func.docstring|docindent(4) -}}

    {%- match func.return_type().ty -%}
    {%- when Some(return_type) %}
    return {{ return_type|lift_fn }}(
        {% if func.is_async() %}await {% endif %}{{ func|ffi_caller_name }}({{ func|arg_names }})
    )
    {% when None %}
    {% if func.is_async() %}await {% endif %}{{ func|ffi_caller_name }}({{ func|arg_names }})
    {%- endmatch %}
