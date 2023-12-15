{%- if func.is_async() %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    return uniffi_rust_call_async(
        UniffiLib.{{ func.ffi_func().name() }}({% call py::arg_list_lowered(func) %}),
        UniffiLib.{{func.ffi_rust_future_poll(ci) }},
        UniffiLib.{{func.ffi_rust_future_complete(ci) }},
        UniffiLib.{{func.ffi_rust_future_free(ci) }},
        # lift function
        {%- match func.return_type() %}
        {%- when Some(return_type) %}
        {{ return_type|lift_fn }},
        {%- when None %}
        lambda val: None,
        {% endmatch %}
        # Error FFI converter
        {%- match func.throws_type() %}
        {%- when Some(e) %}
        {{ e|ffi_converter_name }},
        {%- when None %}
        None,
        {%- endmatch %}
    )

{%- else %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}) -> "{{ return_type|type_name }}":
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    return {{ return_type|lift_fn }}({% call py::to_ffi_call(func) %})
{% when None %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    {% call py::to_ffi_call(func) %}
{% endmatch %}
{%- endif %}
