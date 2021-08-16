class FfiConverter:
    """
    Convert between python objects and "FFI types" (values that we send over the FFI boundary)

    Our naming convention for FfiConverters is `FfiConverter{ CanonicalName }`.

    FfiConverters only have static/class methods.  We never actually create an instance of them.
    """

    @staticmethod
    def lift(value):
        """
        Convert a FFI type to a Python object
        """
        raise NotImplementedError()

    @staticmethod
    def lower(value):
        """
        Convert a Python object to a FFI type
        """
        raise NotImplementedError()

    @staticmethod
    def read(stream):
        """
        Read the type from a RustBufferStream
        """
        raise NotImplementedError()

    @staticmethod
    def write(builder, v):
        """
        Write the type to a RustBufferStream
        """
        raise NotImplementedError()

{%- for typ in ci.iter_types() -%}
{%- match typ -%}
{% when Type::Int8 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(1, ">b")

    @staticmethod
    def write(builder, v):
        builder._pack_into(1, ">b", v)

{% when Type::UInt8 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(1, ">B")

    @staticmethod
    def write(builder, v):
        builder._pack_into(1, ">B", v)

{% when Type::Int16 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(2, ">h")

    @staticmethod
    def write(builder, v):
        builder._pack_into(2, ">h", v)

{% when Type::UInt16 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(2, ">H")

    @staticmethod
    def write(builder, v):
        builder._pack_into(2, ">H", v)

{% when Type::Int32 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(4, ">i")

    @staticmethod
    def write(builder, v):
        builder._pack_into(4, ">i", v)

{% when Type::UInt32 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(4, ">I")

    @staticmethod
    def write(builder, v):
        builder._pack_into(4, ">I", v)

{% when Type::Int64 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(8, ">q")

    @staticmethod
    def write(builder, v):
        builder._pack_into(8, ">q", v)

{% when Type::UInt64 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(int)
    lower = staticmethod(int)

    @staticmethod
    def read(stream):
        return stream._unpack_from(8, ">Q")

    @staticmethod
    def write(builder, v):
        builder._pack_into(8, ">Q", v)

{% when Type::Float32 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(float)
    lower = staticmethod(float)

    @staticmethod
    def read(stream):
        return stream._unpack_from(4, ">f")

    @staticmethod
    def write(builder, v):
        builder = builder._pack_into(4, ">f", v)

{% when Type::Float64 -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(float)
    lower = staticmethod(float)

    @staticmethod
    def read(stream):
        return stream._unpack_from(8, ">d")

    @staticmethod
    def write(builder, v):
        builder._pack_into(8, ">d", v)

{% when Type::Boolean -%}
class {{ typ|ffi_converter_name }}:
    lift = staticmethod(bool)

    @staticmethod
    def lower(value):
        return 1 if value else 0

    @staticmethod
    def read(stream):
        v = stream._unpack_from(1, ">b")
        if v == 0:
            return False
        if v == 1:
            return True
        raise InternalError("Unexpected byte for Boolean type")

    @staticmethod
    def write(builder, v):
        builder._pack_into(1, ">b", 1 if v else 0)

{% when Type::String -%}
class {{ typ|ffi_converter_name }}:
    @staticmethod
    def lift(rust_buffer):
        # Note: this is subtly different from read().  When we're lifting from
        # a `RustBuffer` we can avoid reading the size and use the len field on the
        # `RustBuffer` instead
        with rust_buffer.consumeWithStream() as stream:
            return stream.read(rust_buffer.len).decode("utf-8")

    @staticmethod
    def lower(value):
        with RustBuffer.allocWithBuilder() as builder:
            builder.write(value.encode("utf-8"))
            return builder.finalize()

    @staticmethod
    def read(stream):
        size = stream._unpack_from(4, ">i")
        if size < 0:
            raise InternalError("Unexpected negative string length")
        utf8Bytes = stream.read(size)
        return utf8Bytes.decode("utf-8")

    @staticmethod
    def write(builder, v):
        utf8Bytes = v.encode("utf-8")
        builder._pack_into(4, ">i", len(utf8Bytes))
        builder.write(utf8Bytes)

{% when Type::Timestamp -%}
class {{ typ|ffi_converter_name }}:
    # The Timestamp type.
    # There is a loss of precision when converting from Rust timestamps
    # which are accurate to the nanosecond,to Python datetimes which have
    # a variable precision due to the use of float as representation.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        seconds = stream._unpack_from(8, ">q")
        microseconds = stream._unpack_from(4, ">I") / 1000
        # Use fromtimestamp(0) then add the seconds using a timedelta.  This
        # ensures that we get OverflowError rather than ValueError when
        # seconds is too large.
        if seconds >= 0:
            return datetime.datetime.fromtimestamp(0, tz=datetime.timezone.utc) + datetime.timedelta(seconds=seconds, microseconds=microseconds)
        else:
            return datetime.datetime.fromtimestamp(0, tz=datetime.timezone.utc) - datetime.timedelta(seconds=-seconds, microseconds=microseconds)

    @staticmethod
    def write(builder, v):
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
class {{ typ|ffi_converter_name }}:
    # The Duration type.
    # There is a loss of precision when converting from Rust durations
    # which are accurate to the nanosecond,to Python durations which have
    # are only accurate to the microsecond.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        return datetime.timedelta(seconds=stream._unpack_from(8, ">Q"), microseconds=(stream._unpack_from(4, ">I") / 1.0e3))

    @staticmethod
    def write(builder, v):
        seconds = v.seconds + v.days * 24 * 3600
        nanoseconds = v.microseconds * 1000
        if seconds < 0:
            raise ValueError("Invalid duration, must be non-negative")
        builder._pack_into(8, ">Q", seconds)
        builder._pack_into(4, ">I", nanoseconds)

{% when Type::Object with (object_name) -%}
class {{ typ|ffi_converter_name }}:
    # The Object type {{ object_name }}.

    @staticmethod
    def lift(pointer):
        return {{ object_name|class_name_py }}._make_instance_(pointer)

    @staticmethod
    def lower(obj):
        return obj._pointer

    @staticmethod
    def read(stream):
        # The Rust code always expects pointers written as 8 bytes,
        # and will fail to compile if they don't fit in that size.
        pointer = stream._unpack_from(8, ">Q")
        return {{ object_name|class_name_py }}._make_instance_(pointer)

    @staticmethod
    def write(builder, v):
        if not isinstance(v, {{ object_name|class_name_py }}):
            raise TypeError("Expected {{ object_name|class_name_py }} instance, {} found".format(v.__class__.__name__))
        # The Rust code always expects pointers written as 8 bytes,
        # and will fail to compile if they don't fit in that size.
        builder._pack_into(8, ">Q", v._pointer)

{% when Type::Enum with (enum_name) -%}
{%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
class {{ typ|ffi_converter_name }}:
    # The Enum type {{ enum_name }}.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        variant = stream._unpack_from(4, ">i")
        {% if e.is_flat() -%}
        return {{ enum_name|class_name_py }}(variant)
        {%- else -%}
        {%- for variant in e.variants() %}
        if variant == {{ loop.index }}:
            {%- if variant.has_fields() %}
            return {{ enum_name|class_name_py }}.{{ variant.name()|enum_name_py }}(
                {%- for field in variant.fields() %}
                {{ field.type_()|ffi_converter_name }}.read(stream),
                {%- endfor %}
            )
            {%- else %}
            return {{ enum_name|class_name_py }}.{{ variant.name()|enum_name_py }}()
            {% endif %}
        {%- endfor %}
        raise InternalError("Unexpected variant tag for {{ typ.canonical_name() }}")
        {%- endif %}

    @staticmethod
    def write(builder, v):
        {%- if e.is_flat() %}
        builder._pack_into(4, ">i", v.value)
        {%- else -%}
        {%- for variant in e.variants() %}
        if v.is_{{ variant.name()|var_name_py }}():
            builder._pack_into(4, ">i", {{ loop.index }})
            {%- for field in variant.fields() %}
            {{ field.type_()|ffi_converter_name }}.write(builder, v.{{ field.name() }}),
            {%- endfor %}
        {%- endfor %}
        {%- endif %}

{% when Type::Error with (error_name) -%}
{%- let e = ci.get_error_definition(error_name).unwrap().wrapped_enum() %}

class {{ typ|ffi_converter_name }}:
    # The Error type {{ error_name }}
    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @staticmethod
    def lower(v):
        raise InternalError("Lowering Error types is not supported")

    # Top-level read method
    @classmethod
    def read(cls, stream):
        variant = stream._unpack_from(4, ">i")
        try:
            read_variant_method = getattr(cls, 'readVariant{}'.format(variant))
        except AttributeError:
            raise InternalError("Unexpected variant value for error {{ typ.canonical_name() }} ({})".format(variant))
        return read_variant_method(stream)

    # Read methods for each individual variants
    {%- for variant in e.variants() %}

    @staticmethod
    def readVariant{{ loop.index}}(stream):
        {%- if variant.has_fields() %}
        return {{ error_name|class_name_py }}.{{ variant.name()|class_name_py }}(
            {%- for field in variant.fields() %}
            {{ field.type_()|ffi_converter_name }}.read(stream),
            {%- endfor %}
        )
        {%- else %}
        return {{ error_name|class_name_py }}.{{ variant.name()|class_name_py }}()
        {%- endif %}
    {%- endfor %}

    @staticmethod
    def write(builder, v):
        raise InternalError("Writing Error types is not supported")

{% when Type::Record with (record_name) -%}
{%- let rec = ci.get_record_definition(record_name).unwrap() -%}
class {{ typ|ffi_converter_name }}:
    # The Record type {{ record_name }}.
    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        return {{ rec.name()|class_name_py }}(
            {%- for field in rec.fields() %}
            {{ field.type_()|ffi_converter_name }}.read(stream),
            {%- endfor %}
        )

    @staticmethod
    def write(builder, v):
        {%- for field in rec.fields() %}
        {{ field.type_()|ffi_converter_name }}.write(builder, v.{{ field.name() }})
        {%- endfor %}

{% when Type::Optional with (inner_type) -%}
class {{ typ|ffi_converter_name }}:
    # The Optional<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        flag = stream._unpack_from(1, ">b")
        if flag == 0:
            return None
        elif flag == 1:
            return {{ inner_type|ffi_converter_name }}.read(stream)
        else:
            raise InternalError("Unexpected flag byte for {{ typ.canonical_name() }}")

    @staticmethod
    def write(builder, v):
        if v is None:
            builder._pack_into(1, ">b", 0)
        else:
            builder._pack_into(1, ">b", 1)
            {{ inner_type|ffi_converter_name }}.write(builder, v)

{% when Type::Sequence with (inner_type) -%}
class {{ typ|ffi_converter_name }}:
    # The Sequence<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        count = stream._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative sequence length")
        items = []
        while count > 0:
            items.append({{ inner_type|ffi_converter_name }}.read(stream))
            count -= 1
        return items

    @staticmethod
    def write(builder, items):
        builder._pack_into(4, ">i", len(items))
        for item in items:
            {{ inner_type|ffi_converter_name }}.write(builder, item)

{% when Type::Map with (inner_type) -%}
class {{ typ|ffi_converter_name }}:
    # The Map<T> type for {{ inner_type.canonical_name() }}.

    @classmethod
    def lift(cls, rust_buffer):
        with rust_buffer.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, v):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(builder, v)
            return builder.finalize()

    @staticmethod
    def read(stream):
        count = stream._unpack_from(4, ">i")
        if count < 0:
            raise InternalError("Unexpected negative map size")
        items = {}
        while count > 0:
            key = FfiConverterString.read(stream)
            items[key] = {{ inner_type|ffi_converter_name }}.read(stream)
            count -= 1
        return items

    @staticmethod
    def write(builder, items):
        builder._pack_into(4, ">i", len(items))
        for (k, v) in items.items():
            FfiConverterString.write(builder, k)
            {{ inner_type|ffi_converter_name }}.write(builder, v)

{% when Type::Wrapped with { name, prim } -%}
class {{ typ|ffi_converter_name }}:

    @staticmethod
    def lift(value):
        return {{ prim|ffi_converter_name }}.lift(value)

    @staticmethod
    def lower(value):
        return {{ prim|ffi_converter_name }}.lower(value)

    @staticmethod
    def read(stream):
        return {{ prim|ffi_converter_name }}.read(stream)

    @staticmethod
    def write(builder, value):
        {{ prim|ffi_converter_name }}.write(builder, value)

{% when Type::External with { name, crate_name } -%}
class {{ typ|ffi_converter_name }}:
    @staticmethod
    def lift(value):
        from {{ crate_name|mod_name_py }} import {{ typ|ffi_converter_name }} as ExternalFfiConverter
        return ExternalFfiConverter.lift(value)

    @staticmethod
    def lower(value):
        from {{ crate_name|mod_name_py }} import {{ typ|ffi_converter_name }} as ExternalFfiConverter
        return ExternalFfiConverter.lower(value)

    @staticmethod
    def read(stream):
        from {{ crate_name|mod_name_py }} import {{ typ|ffi_converter_name }} as ExternalFfiConverter
        return ExternalFfiConverter.read(stream)

    @staticmethod
    def write(builder, value):
        from {{ crate_name|mod_name_py }} import {{ typ|ffi_converter_name }} as ExternalFfiConverter
        ExternalFfiConverter.write(builder, value)

    {%- else %}
    # This type is not currently handled, but we can produce a helpful error.
class {{ typ|ffi_converter_name }}:
    @staticmethod
    def lift(value):
        raise InternalError("lift() not implemented yet for {{ typ.canonical_name() }}")

    @staticmethod
    def lower(value):
        raise InternalError("lower() not implemented yet for {{ typ.canonical_name() }}")

    @staticmethod
    def read(stream)
        raise InternalError("read() not implemented yet for {{ typ.canonical_name() }}")

    @staticmethod
    def write(stream)
        raise InternalError("write() not implemented yet for {{ typ.canonical_name() }}")

{%- endmatch -%}
{%- endfor %}
