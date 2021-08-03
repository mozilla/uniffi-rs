
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

    def consumeIntoString(self):
        with self.consumeWithStream() as stream:
            return stream.read(stream.remaining()).decode("utf-8")

    {% when Type::Timestamp -%}

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Duration -%}

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Enum with (enum_name) -%}
    {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
    # The Enum type {{ enum_name }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def allocFrom{{ canonical_type_name }}(v):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write{{ canonical_type_name }}(v)
            return builder.finalize()

    def consumeInto{{ canonical_type_name }}(self):
        with self.consumeWithStream() as stream:
            return stream.read{{ canonical_type_name }}()

    {%- else -%}
    {#- No code emitted for types that don't lower into a RustBuffer -#}
    {%- endmatch -%}
    {%- endfor %}


class ForeignBytes(ctypes.Structure):
    _fields_ = [
        ("len", ctypes.c_int32),
        ("data", ctypes.POINTER(ctypes.c_char)),
    ]

    def __str__(self):
        return "ForeignBytes(len={}, data={})".format(self.len, self.data[0:self.len])
