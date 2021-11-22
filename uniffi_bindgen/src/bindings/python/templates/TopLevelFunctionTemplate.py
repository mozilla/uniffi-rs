{% import "macros.py" as py %}
{%- let func = self.inner() %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::coerce_args(func) %}
    _retval = {% call py::to_ffi_call(func) %}
    return {{ "_retval"|lift_py(return_type) }}

{% when None -%}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::coerce_args(func) %}
    {% call py::to_ffi_call(func) %}
{% endmatch %}
