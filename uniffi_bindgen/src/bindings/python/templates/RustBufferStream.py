
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

class RustBufferTypeReader(object):
    # For every type used in the interface, we provide helper methods for conveniently
    # reading that type in a buffer. Putting them on this internal helper object (rather
    # than, say, as methods on the public classes) makes it easier for us to hide these
    # implementation details from consumers, in the face of python's free-for-all type
    # system.
    # This class holds the logic for *how* to read the types from a buffer - the buffer itself is
    # always passed in, because the actual buffer might be owned by a different crate/module.

    {%- for typ in ci.iter_types() -%}
    {%- let canonical_type_name = typ.canonical_name()|class_name_py -%}
    {%- match typ -%}

    {% when Type::Int8 -%}

    @staticmethod
    def readI8(stream):
        return stream._unpack_from(1, ">b")

    {% when Type::UInt8 -%}

    @staticmethod
    def readU8(stream):
        return stream._unpack_from(1, ">B")

    {% when Type::Int16 -%}

    @staticmethod
    def readI16(stream):
        return stream._unpack_from(2, ">h")

    {% when Type::UInt16 -%}

    @staticmethod
    def readU16(stream):
        return stream._unpack_from(2, ">H")

    {% when Type::Int32 -%}

    @staticmethod
    def readI32(stream):
        return stream._unpack_from(4, ">i")

    {% when Type::UInt32 -%}

    @staticmethod
    def readU32(stream):
        return stream._unpack_from(4, ">I")

    {% when Type::Int64 -%}

    @staticmethod
    def readI64(stream):
        return stream._unpack_from(8, ">q")

    {% when Type::UInt64 -%}

    @staticmethod
    def readU64(stream):
        return stream._unpack_from(8, ">Q")

    {% when Type::Float32 -%}

    @staticmethod
    def readF32(stream):
        v = stream._unpack_from(4, ">f")
        return v

    {% when Type::Float64 -%}

    @staticmethod
    def readF64(stream):
        return stream._unpack_from(8, ">d")

    {% when Type::Boolean -%}

    @staticmethod
    def readBool(stream):
        v = stream._unpack_from(1, ">b")
        if v == 0:
            return False
        if v == 1:
            return True
        raise InternalError("Unexpected byte for Boolean type")

    {% when Type::String -%}

    @staticmethod
    def readString(stream):
        size = stream._unpack_from(4, ">i")
        if size < 0:
            raise InternalError("Unexpected negative string length")
        utf8Bytes = stream.read(size)
        return utf8Bytes.decode("utf-8")


    {% when Type::Timestamp -%}
    # The Timestamp type.
    # There is a loss of precision when converting from Rust timestamps
    # which are accurate to the nanosecond,to Python datetimes which have
    # a variable precision due to the use of float as representation.

    @staticmethod
    def read{{ canonical_type_name }}(stream):
        seconds = stream._unpack_from(8, ">q")
        microseconds = stream._unpack_from(4, ">I") / 1000
        # Use fromtimestamp(0) then add the seconds using a timedelta.  This
        # ensures that we get OverflowError rather than ValueError when
        # seconds is too large.
        if seconds >= 0:
            return datetime.datetime.fromtimestamp(0, tz=datetime.timezone.utc) + datetime.timedelta(seconds=seconds, microseconds=microseconds)
        else:
            return datetime.datetime.fromtimestamp(0, tz=datetime.timezone.utc) - datetime.timedelta(seconds=-seconds, microseconds=microseconds)

    {% when Type::Duration -%}
    # The Duration type.
    # There is a loss of precision when converting from Rust durations
    # which are accurate to the nanosecond,to Python durations which have
    # are only accurate to the microsecond.

    @staticmethod
    def read{{ canonical_type_name }}(stream):
        return datetime.timedelta(seconds=stream._unpack_from(8, ">Q"), microseconds=(stream._unpack_from(4, ">I") / 1.0e3))

    {% when Type::Object with (object_name) -%}
    # The Object type {{ object_name }}.

    @staticmethod
    def read{{ canonical_type_name }}(stream):
        # The Rust code always expects pointers written as 8 bytes,
        # and will fail to compile if they don't fit in that size.
        pointer = stream._unpack_from(8, ">Q")
        return {{ object_name|class_name_py }}._make_instance_(pointer)

    {% when Type::Enum with (enum_name) -%}
    {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
    # The Enum type {{ enum_name }}.

    @staticmethod
    def read{{ canonical_type_name }}(stream):
        variant = stream._unpack_from(4, ">i")
        {% if e.is_flat() -%}
        return {{ enum_name|class_name_py }}(variant)
        {%- else -%}
        {%- for variant in e.variants() %}
        if variant == {{ loop.index }}:
            {%- if variant.has_fields() %}
            return {{ enum_name|class_name_py }}.{{ variant.name()|enum_name_py }}(
                {%- for field in variant.fields() %}
                stream.read{{ field.type_().canonical_name()|class_name_py }}(){% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
            )
            {%- else %}
            return {{ enum_name|class_name_py }}.{{ variant.name()|enum_name_py }}()
            {% endif %}
        {%- endfor %}
        raise InternalError("Unexpected variant tag for {{ canonical_type_name }}")
        {%- endif %}

    {% when Type::Error with (error_name) -%}
    {%- let e = ci.get_error_definition(error_name).unwrap().wrapped_enum() %}

    # The Error type {{ error_name }}

    # Top-level read method
    @classmethod
    def read{{ canonical_type_name }}(cls, stream):
        variant = stream._unpack_from(4, ">i")
        try:
            read_variant_method = getattr(cls, 'readVariant{}Of{{canonical_type_name}}'.format(variant))
        except AttributeError:
            raise InternalError("Unexpected variant value for error {{ canonical_type_name }} ({})".format(variant))
        return read_variant_method(stream)

    # Read methods for each individual variants
    {%- for variant in e.variants() %}

    @classmethod
    def readVariant{{ loop.index}}Of{{ canonical_type_name }}(cls, stream):
        {%- if variant.has_fields() %}
        return {{ error_name|class_name_py }}.{{ variant.name()|class_name_py }}(
            {%- for field in variant.fields() %}
            cls.read{{ field.type_().canonical_name()|class_name_py }}(stream),
            {%- endfor %}
        )
        {%- else %}
        return {{ error_name|class_name_py }}.{{ variant.name()|class_name_py }}()
        {%- endif %}
    {%- endfor %}

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    @classmethod
    def read{{ canonical_type_name }}(cls, stream):
        return {{ rec.name()|class_name_py }}(
            {%- for field in rec.fields() %}
            cls.read{{ field.type_().canonical_name()|class_name_py }}(stream){% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def read{{ canonical_type_name }}(cls, stream):
        flag = stream._unpack_from(1, ">b")
        if flag == 0:
            return None
        elif flag == 1:
            return cls.read{{ inner_type.canonical_name()|class_name_py }}(stream)
        else:
            raise InternalError("Unexpected flag byte for {{ canonical_type_name }}")

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    @staticmethod
    def read{{ canonical_type_name }}(stream):
        count = stream._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative sequence length")
        items = []
        while count > 0:
            items.append(stream.read{{ inner_type.canonical_name()|class_name_py }}())
            count -= 1
        return items

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def read{{ canonical_type_name }}(cls, stream):
        count = stream._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative map size")
        items = {}
        while count > 0:
            key = cls.readString(stream)
            items[key] = stream.read{{ inner_type.canonical_name()|class_name_py }}()
            count -= 1
        return items

    {%- else -%}
    # This type cannot currently be serialized, but we can produce a helpful error.
    @staticmethod
    def read{{ canonical_type_name }}(stream):
        raise InternalError("RustBufferStream.read not implemented yet for {{ canonical_type_name }}")

    {%- endmatch -%}
    {%- endfor %}
