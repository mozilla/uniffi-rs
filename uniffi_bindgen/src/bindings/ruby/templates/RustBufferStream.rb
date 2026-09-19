# Mixin containing type-specific read methods as module functions.
# Call sites name this module explicitly (e.g.
# `::ThisNs::RustBufferStreamMixin.read_TypeFoo(stream)`) so two crates
# that both define `Foo` do not flatten `read_TypeFoo` onto one receiver.
# The `stream` argument is always the *local* crate's RustBufferStream so
# unpack_from / read route through that crate's buffer.
#
# Nested types from further crates are reached by lexical calls in this
# mixin's generated bodies, not by `include`. Do not use `module_function`.
# InternalError in these methods is this crate's class.
module RustBufferStreamMixin
  {% for typ in ci.iter_local_types() -%}
  {%- let canonical_type_name = self::canonical_name(typ) -%}
  {%- match typ -%}

  {% when Type::Int8 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 1, 'c'
  end

  {% when Type::UInt8 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 1, 'c'
  end

  {% when Type::Int16 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 2, 's>'
  end

  {% when Type::UInt16 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 2, 'S>'
  end

  {% when Type::Int32 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 4, 'l>'
  end

  {% when Type::UInt32 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 4, 'L>'
  end

  {% when Type::Int64 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 8, 'q>'
  end

  {% when Type::UInt64 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 8, 'Q>'
  end

  {% when Type::Float32 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 4, 'g'
  end

  {% when Type::Float64 -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    stream.unpack_from 8, 'G'
  end

  {% when Type::Boolean -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    v = stream.unpack_from 1, 'c'

    return false if v == 0
    return true if v == 1

    raise InternalError, 'Unexpected byte for Boolean type'
  end

  {% when Type::String -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    size = stream.unpack_from 4, 'l>'

    raise InternalError, 'Unexpected negative string length' if size.negative?

    stream.read(size).force_encoding(Encoding::UTF_8)
  end

  {% when Type::Bytes -%}

  def self.read_{{ self::canonical_name(typ) }}(stream)
    size = stream.unpack_from 4, 'l>'

    raise InternalError, 'Unexpected negative byte string length' if size.negative?

    stream.read(size).force_encoding(Encoding::BINARY)
  end

  {% when Type::Timestamp -%}
  # The Timestamp type.
  ONE_SECOND_IN_NANOSECONDS = 10**9

  def self.read_{{ canonical_type_name }}(stream)
    seconds = stream.unpack_from 8, 'q>'
    nanoseconds = stream.unpack_from 4, 'L>'

    # UniFFi conventions assume that nanoseconds part has to represent nanoseconds portion of
    # duration between epoch and the timestamp moment. Ruby `Time#tv_nsec` returns the number of
    # nanoseconds for the subsecond part, which is sort of opposite to "duration" meaning.
    # Hence we need to convert value returned by `Time#tv_nsec` back and forth with the following
    # logic:
    if seconds < 0 && nanoseconds != 0
      # In order to get duration nsec we shift by 1 second:
      nanoseconds = ONE_SECOND_IN_NANOSECONDS - nanoseconds

      # Then we compensate 1 second shift:
      seconds -= 1
    end

    Time.at(seconds, nanoseconds, :nanosecond, in: '+00:00').utc
  end

  {% when Type::Duration -%}
  # The Duration type.

  def self.read_{{ canonical_type_name }}(stream)
    seconds = stream.unpack_from 8, 'q>'
    nanoseconds = stream.unpack_from 4, 'L>'

    Time.at(seconds, nanoseconds, :nanosecond, in: '+00:00').utc
  end

  {% when Type::Object with { name: object_name, .. } -%}
  # The Object type {{ object_name }}.

  def self.read_{{ canonical_type_name }}(stream)
    handle = stream.unpack_from 8, 'Q>'
    return {{ object_name|class_name_rb }}.uniffi_lift(handle)
  end

  {% when Type::Enum { name, .. } -%}
  {%- let e = ci.get_enum_definition(name).unwrap() -%}
  {% if !ci.is_name_used_as_error(name) %}
  {% let enum_name = name %}
  # The Enum type {{ enum_name }}.

  def self.read_{{ canonical_type_name }}(stream)
    variant = stream.unpack_from 4, 'l>'
    {% if e.is_flat() -%}
    {%- for variant in e.variants() %}
    if variant == {{ loop.index }}
      return {{ enum_name|class_name_rb }}::{{ variant.name()|enum_name_rb }}
    end
    {%- endfor %}

    raise InternalError, 'Unexpected variant tag for {{ canonical_type_name }}'
    {%- else -%}
    {%- for variant in e.variants() %}
    if variant == {{ loop.index }}
        {%- if variant.has_fields() %}
        {%- let named_fields = !variant.fields()[0].name().is_empty() %}
        return {{ enum_name|class_name_rb }}::{{ variant.name()|enum_name_rb }}.new(
            {%- for field in variant.fields() %}
            {% if named_fields %}{{ field.name()|var_name_rb }}: {% endif %}{{ self.rust_buffer_read(field.as_type().borrow())? }}(stream){% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        {%- else %}
        return {{ enum_name|class_name_rb }}::{{ variant.name()|enum_name_rb }}.new
        {% endif %}
    end
    {%- endfor %}
    raise InternalError, 'Unexpected variant tag for {{ canonical_type_name }}'
    {%- endif %}
  end

  {% else %}

  {% let error_name = name %}

  # The Error type {{ error_name }}

  def self.read_{{ canonical_type_name }}(stream)
    variant = stream.unpack_from 4, 'l>'
    {% if e.is_flat() -%}
    {%- for variant in e.variants() %}
    if variant == {{ loop.index }}
      return {{ error_name|class_name_rb }}::{{ variant.name()|class_name_rb }}.new(
        {{ self.rust_buffer_read(&Type::String)? }}(stream)
      )
    end
    {%- endfor %}

    raise InternalError, 'Unexpected variant tag for {{ canonical_type_name }}'
    {%- else -%}
    {%- for variant in e.variants() %}
    if variant == {{ loop.index }}
        {%- if variant.has_fields() %}
        {%- let named_fields = !variant.fields()[0].name().is_empty() %}
        return {{ error_name|class_name_rb }}::{{ variant.name()|class_name_rb }}.new(
            {%- for field in variant.fields() %}
            {% if named_fields %}{{ field.name()|var_name_rb }}: {% endif %}{{ self.rust_buffer_read(field.as_type().borrow())? }}(stream){% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        {%- else %}
        return {{ error_name|class_name_rb }}::{{ variant.name()|class_name_rb }}.new
        {%- endif %}
    end
    {%- endfor %}

    raise InternalError, 'Unexpected variant tag for {{ canonical_type_name }}'
    {%- endif %}
  end
  {% endif %}

  {% when Type::Record { name: record_name, .. } -%}
  {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
  # The Record type {{ record_name }}.

  def self.read_{{ canonical_type_name }}(stream)
    {{ rec.name()|class_name_rb }}.new(
      {%- for field in rec.fields() %}
      {{ field.name()|var_name_rb }}: {{ self.rust_buffer_read(field.as_type().borrow())? }}(stream){% if loop.last %}{% else %},{% endif %}
      {%- endfor %}
    )
  end

  {% when Type::Optional { inner_type } %}
  # The Optional<T> type for {{ self::canonical_name(inner_type) }}.

  def self.read_{{ canonical_type_name }}(stream)
    flag = stream.unpack_from 1, 'c'

    if flag == 0
      return nil
    elsif flag == 1
      return {{ self.rust_buffer_read(inner_type)? }}(stream)
    else
      raise InternalError, 'Unexpected flag byte for {{ canonical_type_name }}'
    end
  end

  {% when Type::Sequence { inner_type } -%}
  # The Sequence<T> type for {{ self::canonical_name(inner_type) }}.

  def self.read_{{ canonical_type_name }}(stream)
    count = stream.unpack_from 4, 'l>'

    raise InternalError, 'Unexpected negative sequence length' if count.negative?

    items = []

    count.times do
      items.append {{ self.rust_buffer_read(inner_type)? }}(stream)
    end

    items
  end

  {% when Type::Set { inner_type } -%}
  # The Set<T> type for {{ self::canonical_name(inner_type) }}.

  def self.read_{{ canonical_type_name }}(stream)
    count = stream.unpack_from 4, 'l>'

    raise InternalError, 'Unexpected negative set size' if count.negative?

    items = Set.new

    count.times do
      items.add {{ self.rust_buffer_read(inner_type)? }}(stream)
    end

    items
  end

  {% when Type::Map { key_type: k, value_type: v } -%}
  # The Map<T> type for {{ canonical_type_name }}.

  def self.read_{{ canonical_type_name }}(stream)
    count = stream.unpack_from 4, 'l>'
    raise InternalError, 'Unexpected negative map size' if count.negative?

    items = {}
    count.times do
      key = {{ self.rust_buffer_read(k)? }}(stream)
      items[key] = {{ self.rust_buffer_read(v)? }}(stream)
    end

    items
  end

  {% when Type::Custom { name, builtin, .. } -%}
  {%- match config.custom_types.get(name.as_str()) %}
  {%- when Some(cfg) %}{%- if cfg.has_conversion() %}
  # Custom type {{ name }}: reads builtin `{{ self::canonical_name(builtin) }}`, then applies lift.
  def self.read_{{ canonical_type_name }}(stream)
    raw = {{ self.rust_buffer_read(builtin)? }}(stream)
    {{ cfg.lift("raw") }}
  end
  {%- else %}
  # The Custom type {{ name }} delegates deserialization to its builtin type.
  def self.read_{{ canonical_type_name }}(stream)
    {{ self.rust_buffer_read(builtin)? }}(stream)
  end
  {%- endif %}
  {%- when None %}
  # The Custom type {{ name }} delegates deserialization to its builtin type.
  def self.read_{{ canonical_type_name }}(stream)
    {{ self.rust_buffer_read(builtin)? }}(stream)
  end
  {%- endmatch %}

  {% when Type::CallbackInterface { name, .. } -%}

  # The CallbackInterface type {{ name }}: read a uint64 handle.
  def self.read_{{ canonical_type_name }}(stream)
    handle = stream.unpack_from 8, 'Q>'
    {{ self::canonical_name(typ) }}FfiConverter.lift handle
  end

  {%- else -%}
  # This type is not yet supported in the Ruby backend.
  def self.read_{{ canonical_type_name }}(stream)
    raise InternalError, 'RustBufferStream.read not implemented yet for {{ canonical_type_name }}'
  end

  {%- endmatch -%}
  {%- endfor %}
end

# Helper for structured reading of values from a RustBuffer.
class RustBufferStream
  def initialize(rbuf)
    @rbuf = rbuf
    @offset = 0
  end

  def remaining
    @rbuf.len - @offset
  end

  def read(size)
    raise InternalError, 'read past end of rust buffer' if @offset + size > @rbuf.len

    data = @rbuf.data.get_bytes @offset, size

    @offset += size

    data
  end

  def unpack_from(size, format)
    raise InternalError, 'read past end of rust buffer' if @offset + size > @rbuf.len

    value = @rbuf.data.get_bytes(@offset, size).unpack format

    @offset += size

    # TODO: verify this
    raise 'more than one element!!!' if value.size > 1

    value[0]
  end
end

private_constant :RustBufferStream
