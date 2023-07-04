# FFI code for the ForeignExecutor type

{{ self.add_import("asyncio") }}

class {{ ffi_converter_name }}:
    _pointer_manager = _UniffiPointerManager()

    @classmethod
    def lower(cls, eventloop):
        if not isinstance(eventloop, asyncio.BaseEventLoop):
            raise TypeError("_uniffi_executor_callback: Expected EventLoop instance")
        return cls._pointer_manager.new_pointer(eventloop)

    @classmethod
    def write(cls, eventloop, buf):
        buf.write_c_size_t(cls.lower(eventloop))

    @classmethod
    def read(cls, buf):
        return cls.lift(buf.read_c_size_t())

    @classmethod
    def lift(cls, value):
        return cls._pointer_manager.lookup(value)

@_UNIFFI_FOREIGN_EXECUTOR_CALLBACK_T
def _uniffi_executor_callback(eventloop_address, delay, task_ptr, task_data):
    if task_ptr is None:
        {{ ffi_converter_name }}._pointer_manager.release_pointer(eventloop_address)
    else:
        eventloop = {{ ffi_converter_name }}._pointer_manager.lookup(eventloop_address)
        callback = _UNIFFI_RUST_TASK(task_ptr)
        if delay == 0:
            # This can be called from any thread, so make sure to use `call_soon_threadsafe'
            eventloop.call_soon_threadsafe(callback, task_data)
        else:
            # For delayed tasks, we use `call_soon_threadsafe()` + `call_later()` to make the
            # operation threadsafe
            eventloop.call_soon_threadsafe(eventloop.call_later, delay / 1000.0, callback, task_data)

# Register the callback with the scaffolding
_UniffiLib.uniffi_foreign_executor_callback_set(_uniffi_executor_callback)
