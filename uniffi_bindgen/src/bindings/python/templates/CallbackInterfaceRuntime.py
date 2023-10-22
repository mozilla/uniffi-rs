# Magic numbers for the Rust proxy to call using the same mechanism as every other method.

# Dec-ref the callback object
IDX_CALLBACK_FREE = 0
# Inc-ref the callback object
IDX_CALLBACK_CLONE = 0x7FFF_FFFF

# Return codes for callback calls
_UNIFFI_CALLBACK_SUCCESS = 0
_UNIFFI_CALLBACK_ERROR = 1
_UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

class UniffiCallbackInterfaceFfiConverter:
    _handle_map = UniffiHandleMap()

    @classmethod
    def lift(cls, handle):
        return cls._handle_map.get(handle)

    @classmethod
    def read(cls, buf):
        handle = buf.read_u64()
        cls.lift(handle)

    @classmethod
    def check(cls, cb):
        pass

    @classmethod
    def lower(cls, cb):
        handle = cls._handle_map.new_handle(cb)
        return handle

    @classmethod
    def write(cls, cb, buf):
        buf.write_u64(cls.lower(cb))
