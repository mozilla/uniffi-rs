{%- if func.is_async() %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::docstring(func, 4) %}
    {%- call py::setup_args(func) %}
    {# Get the `RustFuture`. -#}
    rust_future = {% call py::to_ffi_call(func) %}
    future = None

    def trampoline() -> (FuturePoll, any):
        nonlocal rust_future

        {% match func.ffi_func().return_type() -%}
        {%- when Some with (return_type) -%}
        polled_result = {{ return_type|ffi_type_name }}()
        polled_result_ref = ctypes.byref(polled_result)
        {%- when None -%}
        polled_result_ref = ctypes.c_void_p()
        {%- endmatch %}

        is_ready = {% match func.throws_type() -%}
        {%- when Some with (error) -%}
        rust_call_with_error({{ error|ffi_converter_name }},
        {%- when None -%}
        rust_call(
        {%- endmatch %}
            _UniFFILib.{{ func.ffi_func().name() }}_poll,
            rust_future,
            future._future_ffi_waker(),
            ctypes.c_void_p(),
            polled_result_ref,
        )

        if is_ready is True:
            result = {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|lift_fn }}(polled_result){% when None %}None{% endmatch %}

            return (FuturePoll.DONE, result)
        else:
            return (FuturePoll.PENDING, None)

    {# Create our own Python `Future` and poll it. -#}
    future = Future(trampoline)
    future.init()

    {# Let's wait on it. -#}
    result = await future

    {# Drop the `rust_future`. -#}
    rust_call(_UniFFILib.{{ func.ffi_func().name() }}_drop, rust_future)

    return result
{%- else %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
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
