# This is a helper for safely working with byte buffers returned from the Rust code.
# It's basically a wrapper around a length and a data pointer, corresponding to the
# `ffi_support::ByteBuffer` struct on the rust side.

class RustBuffer(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_long),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    @staticmethod
    def alloc(size):
        return _UniFFILib.{{ ci.ffi_bytebuffer_alloc().name() }}(size)

    def free(self):
        return _UniFFILib.{{ ci.ffi_bytebuffer_free().name() }}(self)

    def __str__(self):
        return "RustBuffer(len={}, data={})".format(self.len, self.data[0:self.len])