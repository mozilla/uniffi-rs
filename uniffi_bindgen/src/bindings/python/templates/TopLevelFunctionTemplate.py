{%- if func.is_async() %}

{%- match func.return_type() -%}
{%- when Some with (return_type) %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}) -> "{{ return_type|type_name }}":
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    return await _uniffi_rust_call_async(
        _UniffiLib.{{ func.ffi_func().name() }}({% call py::arg_list_lowered(func) %}),
        _UniffiLib.{{func.ffi_rust_future_poll(ci) }},
        _UniffiLib.{{func.ffi_rust_future_complete(ci) }},
        _UniffiLib.{{func.ffi_rust_future_free(ci) }},
        # lift function
        {{ return_type|lift_fn }},
        # Error FFI converter
        {%- match func.throws_type() %}
        {%- when Some(e) %}
        {{ e|ffi_converter_name }},
        {%- when None %}
        None,
        {%- endmatch %}
    )

{% when None %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    return await _uniffi_rust_call_async(
        _UniffiLib.{{ func.ffi_func().name() }}({% call py::arg_list_lowered(func) %}),
        _UniffiLib.{{func.ffi_rust_future_poll(ci) }},
        _UniffiLib.{{func.ffi_rust_future_complete(ci) }},
        _UniffiLib.{{func.ffi_rust_future_free(ci) }},
        # lift function
        lambda val: None,
        # Error FFI converter
        {%- match func.throws_type() %}
        {%- when Some(e) %}
        {{ e|ffi_converter_name }},
        {%- when None %}
        None,
        {%- endmatch %}
    )
{% endmatch %}

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
