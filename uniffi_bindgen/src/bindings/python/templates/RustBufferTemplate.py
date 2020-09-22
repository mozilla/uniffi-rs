# This is a helper for safely working with byte buffers returned from the Rust code.
# It's basically a wrapper around a length and a data pointer, corresponding to the
# `ffi_support::ByteBuffer` struct on the rust side.

class RustBuffer(ctypes.Structure):
    _fields_ = [
        ("capacity", ctypes.c_int32),
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    @staticmethod
    def alloc(size):
        return rust_call_with_error(InternalError, _UniFFILib.{{ ci.ffi_rustbuffer_alloc().name() }}, size)

    @staticmethod
    def reserve(rbuf, additional):
        return rust_call_with_error(InternalError, _UniFFILib.{{ ci.ffi_rustbuffer_reserve().name() }}, rbuf, additional)

    def free(self):
        return rust_call_with_error(InternalError, _UniFFILib.{{ ci.ffi_rustbuffer_free().name() }}, self)

    def __str__(self):
        return "RustBuffer(capacity={}, len={}, data={})".format(
            self.capacity,
            self.len,
            self.data[0:self.len]
        )


class ForeignBytes(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    def __str__(self):
        return "ForeignBytes(len={}, data={})".format(self.len, self.data[0:self.len])