{%- match func.return_type() -%}
{%- when Some with (return_type) %}
{%- if func.is_async() %}

async def {{ func.name()|fn_name }}({%- call py::arg_list_decl(func) -%}):
    {%- call py::setup_args(func) %}

    # Let's call `my_future` on the Rust side. We get a `uniffi::RustFuture` back.
    # Let's store the `uniffi::RustFuture` here maybe?
    ffi_future = None
    future = None
    future_waker = None

    def inner() -> (FuturePoll, any):
        # return {{ return_type|lift_fn }}({% call py::to_ffi_call(func) %})

        # Let's call `uniffi::RustFuture::poll` here.
        # The Waker we pass is a pointer to `def poll`.

        # Handle the result of `uniffi::RustFuture::poll`: if it's done, return
        # `(FuturePoll.DONE, result)`, otherwise `(FuturePoll.PENDING, None)`
        
        print(f'future: {future}')
        print(f'future waker: {future_waker}')

        print('my inner future')

        if True:
            return (FuturePoll.DONE, 42)
        else:
            return (FuturePoll.PENDING, None)

    future = Future(inner)

    # Poll it once.
    future_waker = future._future_waker()
    (future_waker)()

    await future
    
# async def foo() -> None:
#     global waker

#     # Let's call `my_future` on the Rust side. We get a `uniffi::RustFuture` back.
#     # Let's store the `uniffi::RustFuture` here maybe?
    
#     ffifuture = None
#     future = None
#     future_waker = None

#     def my_future() -> (FuturePoll, any):
#         # Let's call `uniffi::RustFuture::poll` here.
#         # The Waker we pass is a pointer to `def poll`.

#         # Handle the result of `uniffi::RustFuture::poll`: if it's done, return
#         # `(FuturePoll.DONE, result)`, otherwise `(FuturePoll.PENDING, None)`
        
#         global hop
#         print(f'future: {future}')
#         print(f'future waker: {future_waker}')

#         print(f'my future: hi {hop}')
        
#         if hop is True:
#             return (FuturePoll.DONE, 42)
#         else:
#             return (FuturePoll.PENDING, None)

#     future = Future(my_future)

#     # Poll it once.
#     future_waker = future._future_waker()
#     (future_waker)()
    
#     # [TEST]
#     waker = future_waker
    
#     await future    

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
