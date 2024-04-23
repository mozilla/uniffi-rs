{{ self.add_import("concurrent.futures") }}

@UNIFFI_BLOCKING_TASK_QUEUE_CLONE
def uniffi_blocking_task_queue_clone(handle):
    executor = _UniffiBlockingTaskQueueHandleMap.get(handle)
    return _UniffiBlockingTaskQueueHandleMap.insert(executor)

@UNIFFI_BLOCKING_TASK_QUEUE_FREE
def uniffi_blocking_task_queue_free(handle):
    _UniffiBlockingTaskQueueHandleMap.remove(handle)

UNIFFI_BLOCKING_TASK_QUEUE_VTABLE = UniffiBlockingTaskQueueVTable(
    uniffi_blocking_task_queue_clone,
    uniffi_blocking_task_queue_free,
)

class {{ ffi_converter_name }}(_UniffiConverterRustBuffer):
    @staticmethod
    def check_lower(value):
        if not isinstance(value, concurrent.futures.Executor):
            raise TypeError("Expected concurrent.futures.Executor instance, {} found".format(type(value).__name__))

    @staticmethod
    def write(value, buf):
        handle = _UniffiBlockingTaskQueueHandleMap.insert(value)
        buf.write_u64(handle)
        buf.write_u64(ctypes.addressof(UNIFFI_BLOCKING_TASK_QUEUE_VTABLE))

    @staticmethod
    def read(buf):
        handle = buf.read_u64()
        executor = _UniffiBlockingTaskQueueHandleMap.remove(handle)
        # Read the VTable pointer and throw it out.  The vtable is only used by Rust and always the
        # same value.
        buf.read_u64()
        return executor

