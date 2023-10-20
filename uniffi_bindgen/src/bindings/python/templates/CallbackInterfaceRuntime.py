import threading

# Magic number for the Rust proxy to call using the same mechanism as every other method,
# to free the callback once it's dropped by Rust.
IDX_CALLBACK_FREE = 0
# Return codes for callback calls
_UNIFFI_CALLBACK_SUCCESS = 0
_UNIFFI_CALLBACK_ERROR = 1
_UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

class UniffiCallbackInterfaceFfiConverter:
    _slab = UniffiSlab()

    @classmethod
    def lift(cls, handle):
        return cls._slab.get(handle)

    @classmethod
    def read(cls, buf):
        handle = buf.read_u64()
        cls.lift(handle)

    @classmethod
    def lower(cls, cb):
        handle = cls._slab.insert(cb)
        return handle

    @classmethod
    def write(cls, cb, buf):
        buf.write_u64(cls.lower(cb))
