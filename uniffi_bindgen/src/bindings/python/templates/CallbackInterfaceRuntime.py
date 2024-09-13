import threading

class ConcurrentHandleMap:
    """
    A map where inserting, getting and removing data is synchronized with a lock.
    """

    def __init__(self):
        # type Handle = int
        self.handles = {}  # type: Dict[Handle, Any]

        self._lock = threading.Lock()
        self._current_handle = 0
        self._stride = 1


    def insert(self, obj):
        with self._lock:
            handle = self._current_handle
            self._current_handle += self._stride
            self.handles[handle] = obj
            return handle

    def get(self, handle):
        with self._lock:
            return self.handles.get(handle)

    def remove(self, handle):
        with self._lock:
            if handle in self.handles:
                return self.handles.pop(handle)

# Magic number for the Rust proxy to call using the same mechanism as every other method,
# to free the callback once it's dropped by Rust.
IDX_CALLBACK_FREE = 0
# Return codes for callback calls
_UNIFFI_CALLBACK_SUCCESS = 0
_UNIFFI_CALLBACK_ERROR = 1
_UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

class _UniffiConverterCallbackInterface:
    _handle_map = ConcurrentHandleMap()

    def __init__(self, cb):
        self._foreign_callback = cb

    def drop(self, handle):
        self.__class__._handle_map.remove(handle)

    @classmethod
    def lift(cls, handle):
        obj = cls._handle_map.get(handle)
        if not obj:
            raise InternalError("The object in the handle map has been dropped already")

        return obj

    @classmethod
    def read(cls, buf):
        handle = buf.read_u64()
        cls.lift(handle)

    @classmethod
    def lower(cls, cb):
        handle = cls._handle_map.insert(cb)
        return handle

    @classmethod
    def write(cls, cb, buf):
        buf.write_u64(cls.lower(cb))
