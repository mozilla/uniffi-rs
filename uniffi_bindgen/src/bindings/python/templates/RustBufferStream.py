
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

    # For every type used in the interface, we provide helper methods for conveniently
    # reading that type in a buffer. Putting them on this internal helper object (rather
    # than, say, as methods on the public classes) makes it easier for us to hide these
    # implementation details from consumers, in the face of python's free-for-all type
    # system.

    {%- for typ in ci.iter_types() -%}
    {%- let canonical_type_name = typ.canonical_name()|class_name_py -%}
    {%- match typ -%}

    {% when Type::Int8 -%}

    def readI8(self):
        return self._unpack_from(1, ">b")

    {% when Type::UInt8 -%}

    def readU8(self):
        return self._unpack_from(1, ">B")

    {% when Type::Int16 -%}

    def readI16(self):
        return self._unpack_from(2, ">h")

    {% when Type::UInt16 -%}

    def readU16(self):
        return self._unpack_from(1, ">H")

    {% when Type::Int32 -%}

    def readI32(self):
        return self._unpack_from(4, ">i")

    {% when Type::UInt32 -%}

    def readU32(self):
        return self._unpack_from(4, ">I")

    {% when Type::Int64 -%}

    def readI64(self):
        return self._unpack_from(8, ">q")

    {% when Type::UInt64 -%}

    def readU64(self):
        return self._unpack_from(8, ">Q")

    {% when Type::Float32 -%}

    def readF32(self):
        v = self._unpack_from(4, ">f")
        return v

    {% when Type::Float64 -%}

    def readF64(self):
        return self._unpack_from(8, ">d")

    {% when Type::Boolean -%}

    def readBool(self):
        v = self._unpack_from(1, ">b")
        if v == 0:
            return False
        if v == 1:
            return True
        raise InternalError("Unexpected byte for Boolean type")

    {% when Type::String -%}

    def readString(self):
        size = self._unpack_from(4, ">i")
        if size < 0:
            raise InternalError("Unexpected negative string length")
        utf8Bytes = self.read(size)
        return utf8Bytes.decode("utf-8")

    {% when Type::Object with (object_name) -%}
    # The Object type {{ object_name }}.
    # Objects cannot currently be serialized, but we can produce a helpful error.

    def read{{ canonical_type_name }}(self):
        raise InternalError("RustBufferStream.read not implemented yet for {{ canonical_type_name }}")

    {% when Type::Error with (error_name) -%}
    # The Error type {{ error_name }}.
    # Errors cannot currently be serialized, but we can produce a helpful error.

    def read{{ canonical_type_name }}(self):
        raise InternalError("RustBufferStream.read not implemented yet for {{ canonical_type_name }}")

    {% when Type::Enum with (enum_name) -%}
    # The Enum type {{ enum_name }}.

    def read{{ canonical_type_name }}(self):
        return {{ enum_name|class_name_py }}(
            self._unpack_from(4, ">i")
        )

    {% when Type::Record with (record_name) -%}
    {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
    # The Record type {{ record_name }}.

    def read{{ canonical_type_name }}(self):
        return {{ rec.name()|class_name_py }}(
            {%- for field in rec.fields() %}
            self.read{{ field.type_().canonical_name()|class_name_py }}(){% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )

    {% when Type::Optional with (inner_type) -%}
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    def read{{ canonical_type_name }}(self):
        flag = self._unpack_from(1, ">b")
        if flag == 0:
            return None
        elif flag == 1:
            return self.read{{ inner_type.canonical_name()|class_name_py }}()
        else:
            raise InternalError("Unexpected flag byte for {{ canonical_type_name }}")

    {% when Type::Sequence with (inner_type) -%}
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    def read{{ canonical_type_name }}(self):
        count = self._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative sequence length")
        items = []
        while count > 0:
            items.append(self.read{{ inner_type.canonical_name()|class_name_py }}())
            count -= 1
        return items

    {% when Type::Map with (inner_type) -%}
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    def read{{ canonical_type_name }}(self):
        count = self._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative map size")
        items = {}
        while count > 0:
            key = self.readString()
            items[key] = self.read{{ inner_type.canonical_name()|class_name_py }}()
            count -= 1
        return items

    {%- endmatch -%}
    {%- endfor %}
