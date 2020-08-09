class RustError(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int32),
        ("message", ctypes.POINTER(ctypes.c_char)),
    ]

    def __str__(self):
        return "RustError(code={}, message={})".format(self.code, self.message)

RustErrorPointer = ctypes.POINTER(RustError)
