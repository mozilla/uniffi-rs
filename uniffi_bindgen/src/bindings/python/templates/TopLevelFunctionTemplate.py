{%- match func.return_type() -%}
{%- when Some with (return_type) %}
{%- if func.is_async() %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}

    rust_future = None
    future = None
    future_waker = None

    def trampoline() -> (FuturePoll, any):
        nonlocal rust_future

        if rust_future is None:
            rust_future = {% call py::to_ffi_call(func) %}
            rust_future.set_waker(future_waker)

        poll_result = rust_future.poll()

        if poll_result is FuturePoll.DONE:
            return (FuturePoll.DONE, 42)
        else:
            return (FuturePoll.PENDING, None)

    future = Future(trampoline)
    future_waker = future._future_waker()

    # Poll it once.
    (future_waker)()

    return await future

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
