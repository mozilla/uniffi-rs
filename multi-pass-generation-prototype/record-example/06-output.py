# The non-type specific parts of RustBuffer, RustBufferBuilder, RustBufferStream

class RustBuffer(ctypes.Structure):
    _fields_ = [
        ("capacity", ctypes.c_int32),
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    @staticmethod
    def alloc(size):
        return rust_call(_UniFFILib.LIBRARY_NAME, size)

    @staticmethod
    def reserve(rbuf, additional):
        return rust_call(_UniFFILib.LIBRARY_NAME, rbuf, additional)

    def free(self):
        return rust_call(_UniFFILib.LIBRARY_NAME, self)

    def __str__(self):
        return "RustBuffer(capacity={}, len={}, data={})".format(
            self.capacity,
            self.len,
            self.data[0:self.len]
        )

    @contextlib.contextmanager
    def allocWithBuilder():
        """Context-manger to allocate a buffer using a RustBufferBuilder.

        The allocated buffer will be automatically freed if an error occurs, ensuring that
        we don't accidentally leak it.
        """
        builder = RustBufferBuilder()
        try:
            yield builder
        except:
            builder.discard()
            raise

    @contextlib.contextmanager
    def consumeWithStream(self):
        """Context-manager to consume a buffer using a RustBufferStream.

        The RustBuffer will be freed once the context-manager exits, ensuring that we don't
        leak it even if an error occurs.
        """
        try:
            s = RustBufferStream(self)
            yield s
            if s.remaining() != 0:
                raise RuntimeError("junk data left in buffer after consuming")
        finally:
            self.free()

class RustBufferBuilder(object):
    """Helper for structured writing of values into a RustBuffer."""

    def __init__(self):
        self.rbuf = RustBuffer.alloc(16)
        self.rbuf.len = 0

    def finalize(self):
        rbuf = self.rbuf
        self.rbuf = None
        return rbuf

    def discard(self):
        if self.rbuf is not None:
            rbuf = self.finalize()
            rbuf.free()

    @contextlib.contextmanager
    def _reserve(self, numBytes):
        if self.rbuf.len + numBytes > self.rbuf.capacity:
            self.rbuf = RustBuffer.reserve(self.rbuf, numBytes)
        yield None
        self.rbuf.len += numBytes

    def _pack_into(self, size, format, value):
        with self._reserve(size):
            # XXX TODO: I feel like I should be able to use `struct.pack_into` here but can't figure it out.
            for i, byte in enumerate(struct.pack(format, value)):
                self.rbuf.data[self.rbuf.len + i] = byte

    def write(self, value):
        with self._reserve(len(value)):
            for i, byte in enumerate(value):
                self.rbuf.data[self.rbuf.len + i] = byte

class RustBufferStream(object):
    """Helper for structured reading of values from a RustBuffer."""

    def __init__(self, rbuf):
        self.rbuf = rbuf
        self.offset = 0

    def remaining(self):
        return self.rbuf.len - self.offset

    def _unpack_from(self, size, format):
        if self.offset + size > self.rbuf.len:
            raise InternalError("read past end of rust buffer")
        value = struct.unpack(format, self.rbuf.data[self.offset:self.offset+size])[0]
        self.offset += size
        return value

    def read(self, size):
        if self.offset + size > self.rbuf.len:
            raise InternalError("read past end of rust buffer")
        data = self.rbuf.data[self.offset:self.offset+size]
        self.offset += size
        return data



class FFIConverterPrimitive:
    size = NotImplemented
    pack_fmt = NotImplemented


    @staticmethod
    def lower(builder, value):
        builder._pack_into(self.size, self.pack_fmt, value)

    @staticmethod
    def lift(stream, value):
        return stream._unpack_from(self.size, self.pack_fmt, value)


class FFIConverterI32(FFIConverterPrimitive):
    pack_fmt = >i
    size = 4



TODO


class Foo:
    def __init__(self, a, b):
        self.a = a
        self.b = b



TODO
