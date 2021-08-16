{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func) -%}):
    return {{ return_type|ffi_converter_name }}.lift({% call py::to_ffi_call(func) %})

{% when None -%}

def {{ func.name()|fn_name_py }}({%- call py::arg_list_decl(func) -%}):
    {% call py::to_ffi_call(func) %}
{% endmatch %}
