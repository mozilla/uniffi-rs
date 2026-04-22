import ctypes
import struct

class _UniffiRustBuffer(ctypes.Structure):
    _fields_ = [
        ("capacity", ctypes.c_uint64),
        ("len", ctypes.c_uint64),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    @staticmethod
    def default():
        return _UniffiRustBuffer(0, 0, None)

    @staticmethod
    def alloc(size):
        return _uniffi_rust_call(_UniffiLib.{{ ffi_rustbuffer_alloc.0 }}, size)

    @staticmethod
    def reserve(rbuf, additional):
        return _uniffi_rust_call(_UniffiLib.{{ ffi_rustbuffer_reserve.0 }}, rbuf, additional)

    def free(self):
        return _uniffi_rust_call(_UniffiLib.{{ ffi_rustbuffer_free.0 }}, self)

    def __str__(self):
        return "_UniffiRustBuffer(capacity={}, len={}, data={})".format(
            self.capacity,
            self.len,
            self.data[0:self.len]
        )

    @contextlib.contextmanager
    def alloc_with_builder(*args):
        """Context-manger to allocate a buffer using a _UniffiRustBufferBuilder.

        The allocated buffer will be automatically freed if an error occurs, ensuring that
        we don't accidentally leak it.
        """
        builder = _UniffiRustBufferBuilder()
        try:
            yield builder
        except:
            builder.discard()
            raise

    @contextlib.contextmanager
    def consume_with_stream(self):
        """Context-manager to consume a buffer using a _UniffiRustBufferStream.

        The _UniffiRustBuffer will be freed once the context-manager exits, ensuring that we don't
        leak it even if an error occurs.
        """
        try:
            s = _UniffiRustBufferStream.from_rust_buffer(self)
            yield s
            if s.remaining() != 0:
                raise RuntimeError(f"junk data left in buffer at end of consume_with_stream {s.remaining()}")
        finally:
            self.free()

    @contextlib.contextmanager
    def read_with_stream(self):
        """Context-manager to read a buffer using a _UniffiRustBufferStream.

        This is like consume_with_stream, but doesn't free the buffer afterwards.
        It should only be used with borrowed `_UniffiRustBuffer` data.
        """
        s = _UniffiRustBufferStream.from_rust_buffer(self)
        yield s
        if s.remaining() != 0:
            raise RuntimeError(f"junk data left in buffer at end of read_with_stream {s.remaining()}")

class _UniffiForeignBytes(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    def __str__(self):
        return "_UniffiForeignBytes(len={}, data={})".format(self.len, self.data[0:self.len])


class _UniffiFfiConverterByRefBytes:
    """Zero-copy converter for `&[u8]` / `[ByRef] bytes` arguments.

    Only `lower` and `check_lower` are valid — zero-copy byte buffers only
    flow foreign -> Rust, and only in argument position. `lift`, `read`, and
    `write` have no sound implementation here.

    CPython `bytes` objects are immutable and their internal buffer doesn't
    move for the lifetime of the object; the caller must keep the source
    `bytes` alive for the duration of the FFI call.
    """

    @staticmethod
    def check_lower(value):
        # Tighter than `bytes-like`: `lower` uses `ctypes.c_char_p` which only
        # accepts `bytes`/`None`, so fail fast with a matching check.
        if not isinstance(value, bytes):
            raise TypeError("a bytes object is required, not {!r}".format(type(value).__name__))

    @staticmethod
    def lower(value):
        fb = _UniffiForeignBytes()
        if len(value) == 0:
            fb.len = 0
            fb.data = None
        else:
            fb.len = len(value)
            fb.data = ctypes.cast(ctypes.c_char_p(value), ctypes.POINTER(ctypes.c_char))
        return fb

    @staticmethod
    def lift(value):
        raise NotImplementedError("ByRef bytes cannot be lifted: zero-copy &[u8] only flows foreign->Rust")

    @staticmethod
    def read(buf):
        raise NotImplementedError("ByRef bytes cannot be read from a buffer: zero-copy &[u8] only flows foreign->Rust")

    @staticmethod
    def write(value, buf):
        raise NotImplementedError("ByRef bytes cannot be written to a buffer: zero-copy &[u8] only flows foreign->Rust")


class _UniffiRustBufferStream:
    """
    Helper for structured reading of bytes from a _UniffiRustBuffer
    """

    def __init__(self, data, len):
        self.data = data
        self.len = len
        self.offset = 0

    @classmethod
    def from_rust_buffer(cls, buf):
        return cls(buf.data, buf.len)

    def remaining(self):
        return self.len - self.offset

    def _unpack_from(self, size, format):
        if self.offset + size > self.len:
            raise InternalError("read past end of rust buffer")
        value = struct.unpack(format, self.data[self.offset:self.offset+size])[0]
        self.offset += size
        return value

    def read(self, size):
        if self.offset + size > self.len:
            raise InternalError("read past end of rust buffer")
        data = self.data[self.offset:self.offset+size]
        self.offset += size
        return data

    def read_i8(self):
        return self._unpack_from(1, ">b")

    def read_u8(self):
        return self._unpack_from(1, ">B")

    def read_i16(self):
        return self._unpack_from(2, ">h")

    def read_u16(self):
        return self._unpack_from(2, ">H")

    def read_i32(self):
        return self._unpack_from(4, ">i")

    def read_u32(self):
        return self._unpack_from(4, ">I")

    def read_i64(self):
        return self._unpack_from(8, ">q")

    def read_u64(self):
        return self._unpack_from(8, ">Q")

    def read_float(self):
        v = self._unpack_from(4, ">f")
        return v

    def read_double(self):
        return self._unpack_from(8, ">d")

class _UniffiRustBufferBuilder:
    """
    Helper for structured writing of bytes into a _UniffiRustBuffer.
    """

    def __init__(self):
        self.rbuf = _UniffiRustBuffer.alloc(16)
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
    def _reserve(self, num_bytes):
        if self.rbuf.len + num_bytes > self.rbuf.capacity:
            self.rbuf = _UniffiRustBuffer.reserve(self.rbuf, num_bytes)
        yield None
        self.rbuf.len += num_bytes

    def _pack_into(self, size, format, value):
        with self._reserve(size):
            packed = struct.pack(format, value)
            if size > 0:
                ctypes.memmove(ctypes.addressof(self.rbuf.data.contents) + self.rbuf.len, packed, size)
    
    def write(self, value):
        length = len(value)
        with self._reserve(length):
            if length > 0:
                ctypes.memmove(ctypes.addressof(self.rbuf.data.contents) + self.rbuf.len, value, length)

    def write_i8(self, v):
        self._pack_into(1, ">b", v)

    def write_u8(self, v):
        self._pack_into(1, ">B", v)

    def write_i16(self, v):
        self._pack_into(2, ">h", v)

    def write_u16(self, v):
        self._pack_into(2, ">H", v)

    def write_i32(self, v):
        self._pack_into(4, ">i", v)

    def write_u32(self, v):
        self._pack_into(4, ">I", v)

    def write_i64(self, v):
        self._pack_into(8, ">q", v)

    def write_u64(self, v):
        self._pack_into(8, ">Q", v)

    def write_float(self, v):
        self._pack_into(4, ">f", v)

    def write_double(self, v):
        self._pack_into(8, ">d", v)

    def write_c_size_t(self, v):
        self._pack_into(ctypes.sizeof(ctypes.c_size_t) , "@N", v)
