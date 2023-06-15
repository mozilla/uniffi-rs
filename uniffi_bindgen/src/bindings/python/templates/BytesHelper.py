class FfiConverterBytes(FfiConverterRustBuffer):
    @staticmethod
    def read(buf):
        size = buf.readI32()
        if size < 0:
            raise InternalError("Unexpected negative byte string length")
        return buf.read(size)

    @staticmethod
    def write(value, buf):
        try:
            memoryview(value)
        except TypeError:
            raise TypeError("a bytes-like object is required, not {!r}".format(type(value).__name__))
        buf.writeI32(len(value))
        buf.write(value)
