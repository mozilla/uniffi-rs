# RustFuturePoll values
UNIFFI_RUST_FUTURE_POLL_READY = 0
UNIFFI_RUST_FUTURE_POLL_MAYBE_READY = 1

# Stores futures for uniffi_continuation_callback
UniffiContinuationPointerManager = UniffiPointerManager()

# Continuation callback for async functions
# lift the return value or error and resolve the future, causing the async function to resume.
@UNIFFI_FUTURE_CONTINUATION_T
def uniffi_continuation_callback(future_ptr, poll_code):
    (eventloop, future) = UniffiContinuationPointerManager.release_pointer(future_ptr)
    eventloop.call_soon_threadsafe(uniffi_set_future_result, future, poll_code)

def uniffi_set_future_result(future, poll_code):
    if not future.cancelled():
        future.set_result(poll_code)

async def uniffi_rust_call_async(rust_future, ffi_poll, ffi_complete, ffi_free, lift_func, error_ffi_converter):
    try:
        eventloop = asyncio.get_running_loop()

        # Loop and poll until we see a UNIFFI_RUST_FUTURE_POLL_READY value
        while True:
            future = eventloop.create_future()
            ffi_poll(
                rust_future,
                uniffi_continuation_callback,
                UniffiContinuationPointerManager.new_pointer((eventloop, future)),
            )
            poll_code = await future
            if poll_code == UNIFFI_RUST_FUTURE_POLL_READY:
                break

        return lift_func(
            _rust_call_with_error(error_ffi_converter, ffi_complete, rust_future)
        )
    finally:
        ffi_free(rust_future)
