{%- if func.is_async() %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    rust_future = None
    future = None

    def trampoline() -> (FuturePoll, any):
        nonlocal rust_future

        if rust_future is None:
            rust_future = {% call py::to_ffi_call(func) %}

        poll_result = {% match func.ffi_func().return_type() %}{% when Some with (return_type) %}{{ return_type|ffi_type_name }}{% when None %}None{% endmatch %}()
        is_ready = rust_call(_UniFFILib.{{ func.ffi_func().name() }}_poll, rust_future, future._future_ffi_waker(), ctypes.byref(poll_result))

        if is_ready is True:
            result = {% match func.return_type() %}{% when Some with (return_type) %}{{ return_type|lift_fn }}(poll_result){% when None %}None{% endmatch %}

            return (FuturePoll.DONE, result)
        else:
            return (FuturePoll.PENDING, None)

    # Create our own `Future`.
    future = Future(trampoline)

    # Poll it once.
    future.init()

    # Let's wait on it.
    result = await future

    # Drop the `rust_future`.
    rust_call(_UniFFILib.{{ func.ffi_func().name() }}_drop, rust_future)

    return result
{%- else %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    return {{ return_type|lift_fn }}({% call py::to_ffi_call(func) %})
{% when None %}

def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}
    {% call py::to_ffi_call(func) %}
{% endmatch %}
{%- endif %}
