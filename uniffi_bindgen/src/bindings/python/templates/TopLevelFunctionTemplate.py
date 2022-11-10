{%- match func.return_type() -%}
{%- when Some with (return_type) %}
{%- if func.is_async() %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    def inner() -> (FuturePoll, any):
        # return {{ return_type|lift_fn }}({% call py::to_ffi_call(func) %})
        pass

    future = Future(inner)
    # waker = future._future_waker()
    
    await future

{%- else %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    return {{ return_type|lift_fn }}({% call py::to_ffi_call(func) %})

{%- endif %}
{% when None %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    {% call py::to_ffi_call(func) %}

{% endmatch %}
