
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

class RustBufferTypeBuilder(object):
    # For every type used in the interface, we provide helper methods for conveniently
    # writing values of that type in a buffer. Putting them on this internal helper object
    # (rather than, say, as methods on the public classes) makes it easier for us to hide
    # these implementation details from consumers, in the face of python's free-for-all
    # type system.
    # This class holds the logic for *how* to write the types to a buffer - the buffer itself is
    # always passed in, because the actual buffer might be owned by a different crate/module.

    {%- for typ in ci.iter_types() -%}
    {%- let canonical_type_name = typ.canonical_name()|class_name_py -%}
    {%- match typ -%}

    {% when Type::Int8 -%}

    @staticmethod
    def writeI8(builder, v):
        builder._pack_into(1, ">b", v)

    {% when Type::UInt8 -%}

    @staticmethod
    def writeU8(builder, v):
        builder._pack_into(1, ">B", v)

    {% when Type::Int16 -%}

    @staticmethod
    def writeI16(builder, v):
        builder._pack_into(2, ">h", v)

    {% when Type::UInt16 -%}

    @staticmethod
    def writeU16(builder, v):
        builder._pack_into(2, ">H", v)

    {% when Type::Int32 -%}

    @staticmethod
    def writeI32(builder, v):
        builder._pack_into(4, ">i", v)

    {% when Type::UInt32 -%}

    @staticmethod
    def writeU32(builder, v):
        builder._pack_into(4, ">I", v)

    {% when Type::Int64 -%}

    @staticmethod
    def writeI64(builder, v):
        builder._pack_into(8, ">q", v)

    {% when Type::UInt64 -%}

    @staticmethod
    def writeU64(builder, v):
        builder._pack_into(8, ">Q", v)

    {% when Type::Float32 -%}

    @staticmethod
    def writeF32(builder, v):
        builder._pack_into(4, ">f", v)

    {% when Type::Float64 -%}

    @staticmethod
    def writeF64(builder, v):
        builder._pack_into(8, ">d", v)

    {% when Type::Boolean -%}

    @staticmethod
    def writeBool(builder, v):
        builder._pack_into(1, ">b", 1 if v else 0)

    {% when Type::String -%}

    @staticmethod
    def writeString(builder, v):
        utf8Bytes = v.encode("utf-8")
        builder._pack_into(4, ">i", len(utf8Bytes))
        builder.write(utf8Bytes)

    {% when Type::Timestamp -%}

    @staticmethod
    def write{{ canonical_type_name }}(builder, v):
        if v >= datetime.datetime.fromtimestamp(0, datetime.timezone.utc):
            sign = 1
            delta = v - datetime.datetime.fromtimestamp(0, datetime.timezone.utc)
        else:
            sign = -1
            delta = datetime.datetime.fromtimestamp(0, datetime.timezone.utc) - v

        seconds = delta.seconds + delta.days * 24 * 3600
        nanoseconds = delta.microseconds * 1000
        builder._pack_into(8, ">q", sign * seconds)
        builder._pack_into(4, ">I", nanoseconds)

    {% when Type::Duration -%}

    @staticmethod
    def write{{ canonical_type_name }}(builder, v):
        seconds = v.seconds + v.days * 24 * 3600
        nanoseconds = v.microseconds * 1000
        if seconds < 0:
            raise ValueError("Invalid duration, must be non-negative")
        builder._pack_into(8, ">Q", seconds)
        builder._pack_into(4, ">I", nanoseconds)

    {% when Type::Object with (object_name) -%}
    # The Object type {{ object_name }}.
    # We write the pointer value directly - what could possibly go wrong?

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        if not isinstance(v, {{ object_name|class_name_py }}):
            raise TypeError("Expected {{ object_name|class_name_py }} instance, {} found".format(v.__class__.__name__))
        # The Rust code always expects pointers written as 8 bytes,
        # and will fail to compile if they don't fit in that size.
        cls.writeU64(builder, v._pointer)

    {% when Type::Enum with (enum_name) -%}
    {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
    # The Enum type {{ enum_name }}.

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        {%- if e.is_flat() %}
        builder._pack_into(4, ">i", v.value)
        {%- else -%}
        {%- for variant in e.variants() %}
        if v.is_{{ variant.name()|var_name_py }}():
            builder._pack_into(4, ">i", {{ loop.index }})
            {%- for field in variant.fields() %}
            cls.write{{ field.type_().canonical_name()|class_name_py }}(builder, v.{{ field.name() }})
            {%- endfor %}
        {%- endfor %}
        {%- endif %}

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        {%- for field in rec.fields() %}
        cls.write{{ field.type_().canonical_name()|class_name_py }}(builder, v.{{ field.name() }})
        {%- endfor %}

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        if v is None:
            builder._pack_into(1, ">b", 0)
        else:
            builder._pack_into(1, ">b", 1)
            cls.write{{ inner_type.canonical_name()|class_name_py }}(builder, v)

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, items):
        builder._pack_into(4, ">i", len(items))
        for item in items:
            cls.write{{ inner_type.canonical_name()|class_name_py }}(builder, item)

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, items):
        builder._pack_into(4, ">i", len(items))
        for (k, v) in items.items():
            cls.writeString(builder, k)
            cls.write{{ inner_type.canonical_name()|class_name_py }}(builder, v)

    {% when Type::Wrapped with { name, prim } %}

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        cls.write{{ prim.canonical_name()|class_name_py }}(builder, v)

    {%- when Type::External with { name, crate_name } %}

    @classmethod
    def write{{ canonical_type_name }}(cls, builder, v):
        from {{ crate_name }} import RustBufferTypeBuilder;
        RustBufferTypeBuilder.write{{ canonical_type_name }}(builder, v)

    {%- else -%}
    # This type cannot currently be serialized, but we can produce a helpful error.

    @staticmethod
    def write{{ canonical_type_name }}(self, builder):
        raise InternalError("RustBufferStream.write() not implemented yet for {{ canonical_type_name }}")

    {%- endmatch -%}
    {%- endfor %}
