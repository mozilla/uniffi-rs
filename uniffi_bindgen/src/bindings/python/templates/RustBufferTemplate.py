
class RustBuffer(ctypes.Structure):
    _fields_ = [
        ("capacity", ctypes.c_int32),
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    @staticmethod
    def alloc(size):
        return rust_call(_UniFFILib.{{ ci.ffi_rustbuffer_alloc().name() }}, size)

    @staticmethod
    def reserve(rbuf, additional):
        return rust_call(_UniFFILib.{{ ci.ffi_rustbuffer_reserve().name() }}, rbuf, additional)

    def free(self):
        return rust_call(_UniFFILib.{{ ci.ffi_rustbuffer_free().name() }}, self)

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

    # For every type that lowers into a RustBuffer, we provide helper methods for
    # conveniently doing the lifting and lowering. Putting them on this internal
    # helper object (rather than, say, as methods on the public classes) makes it
    # easier for us to hide these implementation details from consumers, in the face
    # of python's free-for-all type system.

    {%- for typ in ci.iter_types() -%}
    {%- let canonical_type_name = typ.canonical_name() -%}
    {%- match typ -%}

    {% when Type::String -%}
    # The primitive String type.

    @staticmethod
    def allocFromString(value):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write(value.encode("utf-8"))
            return builder.finalize()

    {% when Type::Timestamp -%}

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Duration -%}

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Enum with (enum_name) -%}
    {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
    # The Enum type {{ enum_name }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)
            return builder.finalize()

    {%- else -%}
    {#- No code emitted for types that don't lower into a RustBuffer -#}
    {%- endmatch -%}
    {%- endfor %}

class RustBufferStream(object):
    # Helper for structured reading of bytes from a RustBuffer

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

class RustBufferBuilder(object):
    # Helper for structured writing of bytes into a RustBuffer.

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


class ForeignBytes(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    def __str__(self):
        return "ForeignBytes(len={}, data={})".format(self.len, self.data[0:self.len])
