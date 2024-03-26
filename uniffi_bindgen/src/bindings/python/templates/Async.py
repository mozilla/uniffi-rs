# RustFuturePoll values
_UNIFFI_RUST_FUTURE_POLL_READY = 0
_UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1

"""
Data for an in-progress poll of a RustFuture
"""
class UniffiPoll(typing.NamedTuple):
    eventloop: asyncio.AbstractEventLoop
    future: asyncio.Future
    rust_future: int
    # Must be UNIFFI_RUST_FUTURE_CONTINUATION_CALLBACK, but it's not clear how to specify as valid
    # type for mypy and our current Python version
    ffi_poll: object

# Stores the UniffiPoll instances that correspond to RustFuture callback data
_UniffiPollDataHandleMap = _UniffiHandleMap()

# Stores the concurrent.futures.Executor instances that correspond to blocking task queue handles
_UniffiBlockingTaskQueueHandleMap = _UniffiHandleMap()

UNIFFI_GLOBAL_EVENT_LOOP = None

"""
Set the event loop to use for async functions

This is needed if some async functions run outside of the eventloop, for example:
    - A non-eventloop thread is spawned, maybe from `EventLoop.run_in_executor` or maybe from the
      Rust code spawning its own thread.
    - The Rust code calls an async callback method from a sync callback function, using something
      like `pollster` to block on the async call.

In this case, we need an event loop to run the Python async function, but there's no eventloop set
for the thread.  Use `uniffi_set_event_loop` to force an eventloop to be used in this case.
"""
def uniffi_set_event_loop(eventloop: asyncio.BaseEventLoop):
    global UNIFFI_GLOBAL_EVENT_LOOP
    UNIFFI_GLOBAL_EVENT_LOOP = eventloop

def _uniffi_get_event_loop():
    if UNIFFI_GLOBAL_EVENT_LOOP is not None:
        return UNIFFI_GLOBAL_EVENT_LOOP
    else:
        return asyncio.get_running_loop()

# Continuation callback for async functions
# lift the return value or error and resolve the future, causing the async function to resume.
@UNIFFI_RUST_FUTURE_CONTINUATION_CALLBACK
def _uniffi_continuation_callback(poll_data_handle, poll_code, blocking_task_queue_handle):
    if blocking_task_queue_handle == 0:
        # Complete the Python Future
        poll_data = _UniffiPollDataHandleMap.remove(poll_data_handle)
        poll_data.eventloop.call_soon_threadsafe(_uniffi_set_future_result, poll_data.future, poll_code)
    else:
        # Call the poll function again, but inside the executor
        poll_data = _UniffiPollDataHandleMap.get(poll_data_handle)
        executor = _UniffiBlockingTaskQueueHandleMap.get(blocking_task_queue_handle)
        executor.submit(
            poll_data.ffi_poll,
            poll_data.rust_future,
            _uniffi_continuation_callback,
            poll_data_handle,
            blocking_task_queue_handle
        )

def _uniffi_set_future_result(future, poll_code):
    if not future.cancelled():
        future.set_result(poll_code)

async def _uniffi_rust_call_async(rust_future, ffi_poll, ffi_complete, ffi_free, lift_func, error_ffi_converter):
    try:
        eventloop = _uniffi_get_event_loop()

        # Loop and poll until we see a _UNIFFI_RUST_FUTURE_POLL_READY value
        while True:
            future = eventloop.create_future()
            poll_data = UniffiPoll(
                eventloop=eventloop,
                future=future,
                rust_future=rust_future,
                ffi_poll=ffi_poll,
            )
            ffi_poll(
                rust_future,
                _uniffi_continuation_callback,
                _UniffiPollDataHandleMap.insert(poll_data),
                0,
            )
            poll_code = await future
            if poll_code == _UNIFFI_RUST_FUTURE_POLL_READY:
                break

        return lift_func(
            _rust_call_with_error(error_ffi_converter, ffi_complete, rust_future)
        )
    finally:
        ffi_free(rust_future)

{%- if ci.has_async_callback_interface_definition() %}
def uniffi_trait_interface_call_async(make_call, handle_success, handle_error):
    async def make_call_and_call_callback():
        try:
            handle_success(await make_call())
        except Exception as e:
            print("UniFFI: Unhandled exception in trait interface call", file=sys.stderr)
            traceback.print_exc(file=sys.stderr)
            handle_error(
                _UniffiRustCallStatus.CALL_UNEXPECTED_ERROR,
                {{ Type::String.borrow()|lower_fn }}(repr(e)),
            )
    eventloop = _uniffi_get_event_loop()
    task = asyncio.run_coroutine_threadsafe(make_call_and_call_callback(), eventloop)
    handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert((eventloop, task))
    return UniffiForeignFuture(handle, uniffi_foreign_future_free)

def uniffi_trait_interface_call_async_with_error(make_call, handle_success, handle_error, error_type, lower_error):
    async def make_call_and_call_callback():
        try:
            try:
                handle_success(await make_call())
            except error_type as e:
                handle_error(
                    _UniffiRustCallStatus.CALL_ERROR,
                    lower_error(e),
                )
        except Exception as e:
            print("UniFFI: Unhandled exception in trait interface call", file=sys.stderr)
            traceback.print_exc(file=sys.stderr)
            handle_error(
                _UniffiRustCallStatus.CALL_UNEXPECTED_ERROR,
                {{ Type::String.borrow()|lower_fn }}(repr(e)),
            )
    eventloop = _uniffi_get_event_loop()
    task = asyncio.run_coroutine_threadsafe(make_call_and_call_callback(), eventloop)
    handle = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.insert((eventloop, task))
    return UniffiForeignFuture(handle, uniffi_foreign_future_free)

UNIFFI_FOREIGN_FUTURE_HANDLE_MAP = _UniffiHandleMap()

@UNIFFI_FOREIGN_FUTURE_FREE
def uniffi_foreign_future_free(handle):
    (eventloop, task) = UNIFFI_FOREIGN_FUTURE_HANDLE_MAP.remove(handle)
    eventloop.call_soon(uniffi_foreign_future_do_free, task)

def uniffi_foreign_future_do_free(task):
    if not task.done():
        task.cancel()
{%- endif %}
