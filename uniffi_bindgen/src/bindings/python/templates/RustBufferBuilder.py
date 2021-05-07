
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

    # For every type used in the interface, we provide helper methods for conveniently
    # writing values of that type in a buffer. Putting them on this internal helper object
    # (rather than, say, as methods on the public classes) makes it easier for us to hide
    # these implementation details from consumers, in the face of python's free-for-all
    # type system.

    {%- for typ in ci.iter_types() -%}
    {%- let canonical_type_name = typ.canonical_name()|class_name_py -%}
    {%- match typ -%}

    {% when Type::Int8 -%}

    def writeI8(self, v):
        self._pack_into(1, ">b", v)

    {% when Type::UInt8 -%}

    def writeU8(self, v):
        self._pack_into(1, ">B", v)

    {% when Type::Int16 -%}

    def writeI16(self, v):
        self._pack_into(2, ">h", v)

    {% when Type::UInt16 -%}

    def writeU16(self, v):
        self._pack_into(1, ">H", v)

    {% when Type::Int32 -%}

    def writeI32(self, v):
        self._pack_into(4, ">i", v)

    {% when Type::UInt32 -%}

    def writeU32(self, v):
        self._pack_into(4, ">I", v)

    {% when Type::Int64 -%}

    def writeI64(self, v):
        self._pack_into(8, ">q", v)

    {% when Type::UInt64 -%}

    def writeU64(self, v):
        self._pack_into(8, ">Q", v)

    {% when Type::Float32 -%}

    def writeF32(self, v):
        self._pack_into(4, ">f", v)

    {% when Type::Float64 -%}

    def writeF64(self, v):
        self._pack_into(8, ">d", v)

    {% when Type::Boolean -%}

    def writeBool(self, v):
        self._pack_into(1, ">b", 1 if v else 0)

    {% when Type::String -%}

    def writeString(self, v):
        utf8Bytes = v.encode("utf-8")
        self._pack_into(4, ">i", len(utf8Bytes))
        self.write(utf8Bytes)

    {% when Type::JSONValue -%}

    def writeJsonValue(self, v):
        import json
        json_string = json.dumps(v, separators=(',', ':'))
        self.writeString(json_string)

    {% when Type::Object with (object_name) -%}
    # The Object type {{ object_name }}.
    # Objects cannot currently be serialized, but we can produce a helpful error.

    def write{{ canonical_type_name }}(self):
        raise InternalError("RustBufferStream.write() not implemented yet for {{ canonical_type_name }}")

    {% when Type::CallbackInterface with (object_name) -%}
    # The Callback Interface type {{ object_name }}.
    # Objects cannot currently be serialized, but we can produce a helpful error.

    def write{{ canonical_type_name }}(self):
        raise InternalError("RustBufferStream.write() not implemented yet for {{ canonical_type_name }}")

    {% when Type::Error with (error_name) -%}
    # The Error type {{ error_name }}.
    # Errors cannot currently be serialized, but we can produce a helpful error.

    def write{{ canonical_type_name }}(self):
        raise InternalError("RustBufferStream.write() not implemented yet for {{ canonical_type_name }}")

    {% when Type::Enum with (enum_name) -%}
    {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
    # The Enum type {{ enum_name }}.

    def write{{ canonical_type_name }}(self, v):
        {%- if e.is_flat() %}
        self._pack_into(4, ">i", v.value)
        {%- else -%}
        {%- for variant in e.variants() %}
        if v.is_{{ variant.name()|var_name_py }}():
            self._pack_into(4, ">i", {{ loop.index }})
            {%- for field in variant.fields() %}
            self.write{{ field.type_().canonical_name()|class_name_py }}(v.{{ field.name() }})
            {%- endfor %}
        {%- endfor %}
        {%- endif %}

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    def write{{ canonical_type_name }}(self, v):
        {%- for field in rec.fields() %}
        self.write{{ field.type_().canonical_name()|class_name_py }}(v.{{ field.name() }})
        {%- endfor %}

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    def write{{ canonical_type_name }}(self, v):
        if v is None:
            self._pack_into(1, ">b", 0)
        else:
            self._pack_into(1, ">b", 1)
            self.write{{ inner_type.canonical_name()|class_name_py }}(v)

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    def write{{ canonical_type_name }}(self, items):
        self._pack_into(4, ">i", len(items))
        for item in items:
            self.write{{ inner_type.canonical_name()|class_name_py }}(item)

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    def write{{ canonical_type_name }}(self, items):
        self._pack_into(4, ">i", len(items))
        for (k, v) in items.items():
            self.writeString(k)
            self.write{{ inner_type.canonical_name()|class_name_py }}(v)

    {%- endmatch -%}
    {%- endfor %}