{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func.arguments()) -%}):
    {%- call py::coerce_args(func.arguments()) %}
    _retval = {% call py::to_rs_call(func.ffi_func()) %}
    return {{ "_retval"|lift_py(return_type) }}

{% when None -%}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func.arguments()) -%}):
    {%- call py::coerce_args(func.arguments()) %}
    {% call py::to_rs_call(func.ffi_func()) %}
{% endmatch %}