# Mixin containing type-specific write methods as module functions.
# Call sites name this module explicitly (e.g.
# `::ThisNs::RustBufferBuilderMixin.write_TypeFoo(builder, v)`) so two crates
# that both define `Foo` do not flatten `write_TypeFoo` onto one receiver.
# The `builder` argument is always the *local* crate's RustBufferBuilder so
# pack_into / reserve / write route through that crate's allocator.
#
# Nested types from further crates are reached by lexical calls in this
# mixin's generated bodies, not by `include`. Do not use `module_function`
# (it would reintroduce private instance methods).
# InternalError in these methods is this crate's class.
module RustBufferBuilderMixin
  {% for typ in ci.iter_local_types() -%}
  {%- let canonical_type_name = self::canonical_name(typ) -%}
  {%- match typ -%}

  {% when Type::Int8 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "i8", -2**7, 2**7)
    builder.pack_into(1, 'c', v)
  end

  {% when Type::UInt8 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "u8", 0, 2**8)
    builder.pack_into(1, 'c', v)
  end

  {% when Type::Int16 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "i16", -2**15, 2**15)
    builder.pack_into(2, 's>', v)
  end

  {% when Type::UInt16 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "u16", 0, 2**16)
    builder.pack_into(2, 'S>', v)
  end

  {% when Type::Int32 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "i32", -2**31, 2**31)
    builder.pack_into(4, 'l>', v)
  end

  {% when Type::UInt32 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "u32", 0, 2**32)
    builder.pack_into(4, 'L>', v)
  end

  {% when Type::Int64 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "i64", -2**63, 2**63)
    builder.pack_into(8, 'q>', v)
  end

  {% when Type::UInt64 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_in_range(v, "u64", 0, 2**64)
    builder.pack_into(8, 'Q>', v)
  end

  {% when Type::Float32 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    builder.pack_into(4, 'g', v)
  end

  {% when Type::Float64 -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    builder.pack_into(8, 'G', v)
  end

  {% when Type::Boolean -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    builder.pack_into(1, 'c', v ? 1 : 0)
  end

  {% when Type::String -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_utf8(v)
    builder.pack_into 4, 'l>', v.bytes.size
    builder.write v
  end

  {% when Type::Bytes -%}

  def self.write_{{ canonical_type_name }}(builder, v)
    v = ::{{ self.module_name() }}::uniffi_bytes(v)
    builder.pack_into 4, 'l>', v.bytes.size
    builder.write v
  end

  {% when Type::Timestamp -%}
  # The Timestamp type.
  ONE_SECOND_IN_NANOSECONDS = 10**9

  def self.write_{{ canonical_type_name }}(builder, v)
    seconds = v.tv_sec
    nanoseconds = v.tv_nsec

    # UniFFi conventions assume that nanoseconds part has to represent nanoseconds portion of
    # duration between epoch and the timestamp moment. Ruby `Time#tv_nsec` returns the number of
    # nanoseconds for the subsecond part, which is sort of opposite to "duration" meaning.
    # Hence we need to convert value returned by `Time#tv_nsec` back and forth with the following
    # logic:
    if seconds < 0 && nanoseconds != 0
      # In order to get duration nsec we shift by 1 second:
      nanoseconds = ONE_SECOND_IN_NANOSECONDS - nanoseconds

      # Then we compensate 1 second shift:
      seconds += 1
    end

    builder.pack_into 8, 'q>', seconds
    builder.pack_into 4, 'L>', nanoseconds
  end

  {% when Type::Duration -%}
  # The Duration type.

  def self.write_{{ canonical_type_name }}(builder, v)
    seconds = v.tv_sec
    nanoseconds = v.tv_nsec

    raise ArgumentError, 'Invalid duration, must be non-negative' if seconds < 0

    builder.pack_into 8, 'Q>', seconds
    builder.pack_into 4, 'L>', nanoseconds
  end

  {% when Type::Object with { name: object_name, .. } -%}
  # The Object type {{ object_name }}.

  def self.write_{{ canonical_type_name }}(builder, obj)
    handle = {{ object_name|class_name_rb}}.uniffi_lower obj
    builder.pack_into(8, 'Q>', handle)
  end

  {% when Type::Enum { name: enum_name, .. } -%}
  {% if !ci.is_name_used_as_error(enum_name) %}
  {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
  # The Enum type {{ enum_name }}.

  def self.write_{{ canonical_type_name }}(builder, v)
    {%- if e.is_flat() %}
    {%- for variant in e.variants() %}
    if v == {{ enum_name|class_name_rb }}::{{ variant.name()|enum_name_rb }}
      builder.pack_into(4, 'l>', {{ loop.index }})
    end
    {%- endfor %}
    {%- else -%}
    {%- for variant in e.variants() %}
    if v.{{ variant.name()|var_name_rb }}?
      builder.pack_into(4, 'l>', {{ loop.index }})
      {%- for field in variant.fields() %}
        {{ self.rust_buffer_write(field.as_type().borrow())? }}(builder, v.{% call rb::field_name(field, loop.index) %}{% endcall %})
      {%- endfor %}
    end
    {%- endfor %}
    {%- endif %}
 end
  {% else %}
  {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
  # The Error type {{ enum_name }} - write for callback error returns.

  def self.write_{{ canonical_type_name }}(builder, v)
    {%- if e.is_flat() %}
    {%- for variant in e.variants() %}
    if v.is_a?({{ enum_name|class_name_rb }}::{{ variant.name()|class_name_rb }})
      builder.pack_into 4, 'l>', {{ loop.index }}
      return
    end
    {%- endfor %}
    {%- else -%}
    {%- for variant in e.variants() %}
    if v.is_a?({{ enum_name|class_name_rb }}::{{ variant.name()|class_name_rb }})
      builder.pack_into 4, 'l>', {{ loop.index }}
      {%- for field in variant.fields() %}
        {%- if field.name().is_empty() %}
        {{ self.rust_buffer_write(field.as_type().borrow())? }}(builder, v[{{ loop.index0 }}])
        {%- else %}
        {{ self.rust_buffer_write(field.as_type().borrow())? }}(builder, v.{{ field.name()|var_name_rb }})
        {%- endif %}
      {%- endfor %}
      return
    end
    {%- endfor %}
    {%- endif %}
  end
  {% endif %}

  {% when Type::Record { name: record_name, .. } -%}
  {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
  # The Record type {{ record_name }}.

  def self.write_{{ canonical_type_name }}(builder, v)
    {%- for field in rec.fields() %}
    {{ self.rust_buffer_write(field.as_type().borrow())? }}(builder, v.{{ field.name()|var_name_rb }})
    {%- endfor %}
  end

  {% when Type::Optional { inner_type } -%}
  # The Optional<T> type for {{ self::canonical_name(inner_type) }}.

  def self.write_{{ canonical_type_name }}(builder, v)
    if v.nil?
      builder.pack_into(1, 'c', 0)
    else
      builder.pack_into(1, 'c', 1)
      {{ self.rust_buffer_write(inner_type)? }}(builder, v)
    end
  end

  {% when Type::Sequence { inner_type } -%}
  # The Sequence<T> type for {{ self::canonical_name(inner_type) }}.

  def self.write_{{ canonical_type_name }}(builder, items)
    builder.pack_into(4, 'l>', items.size)

    items.each do |item|
      {{ self.rust_buffer_write(inner_type)? }}(builder, item)
    end
  end

  {% when Type::Set { inner_type } -%}
  # The Set<T> type for {{ self::canonical_name(inner_type) }}.

  def self.write_{{ canonical_type_name }}(builder, items)
    builder.pack_into(4, 'l>', items.size)

    items.each do |item|
      {{ self.rust_buffer_write(inner_type)? }}(builder, item)
    end
  end

  {% when Type::Map { key_type: k, value_type: v } -%}
  # The Map<T> type for {{ canonical_type_name }}.

  def self.write_{{ canonical_type_name }}(builder, items)
    builder.pack_into(4, 'l>', items.size)

    items.each do |k, v|
      {{ self.rust_buffer_write(k)? }}(builder, k)
      {{ self.rust_buffer_write(v)? }}(builder, v)
    end
  end

  {% when Type::Custom { name, builtin, .. } -%}
  {%- match config.custom_types.get(name.as_str()) %}
  {%- when Some(cfg) %}{%- if cfg.has_conversion() %}
  # Custom type {{ name }}: applies lower, then writes builtin `{{ self::canonical_name(builtin) }}`
  def self.write_{{ canonical_type_name }}(builder, v)
    {{ self.rust_buffer_write(builtin)? }}(builder, {{ cfg.lower("v") }})
  end
  {%- else %}
  # The Custom type {{ name }} delegates serialization to its builtin type.
  def self.write_{{ canonical_type_name }}(builder, v)
    {{ self.rust_buffer_write(builtin)? }}(builder, v)
  end
  {%- endif %}
  {%- when None %}
  # The Custom type {{ name }} delegates serialization to its builtin type.
  def self.write_{{ canonical_type_name }}(builder, v)
    {{ self.rust_buffer_write(builtin)? }}(builder, v)
  end
  {%- endmatch %}

  {% when Type::CallbackInterface { name, .. } -%}
  # The CallbackInterface type {{ name }}: write a uint64 handle.
  def self.write_{{ canonical_type_name }}(builder, v)
    handle = {{ self::canonical_name(typ) }}FfiConverter.lower(v)
    builder.pack_into 8, 'Q>', handle
  end

  {%- else -%}
  # This type is not yet supported in the Ruby backend.
  def self.write_{{ canonical_type_name }}(builder, v)
    raise InternalError('RustBufferStream.write() not implemented yet for {{ canonical_type_name }}')
  end

  {%- endmatch -%}
  {%- endfor %}
end

# Helper for structured writing of values into a RustBuffer.
class RustBufferBuilder
  def initialize
    @rust_buf = RustBuffer.alloc 16
    @rust_buf.len = 0
  end

  def finalize
    rbuf = @rust_buf

    @rust_buf = nil

    rbuf
  end

  def discard
    return if @rust_buf.nil?

    rbuf = finalize
    rbuf.free
  end

  def write(value)
    reserve(value.bytes.size) do
      @rust_buf.data.put_array_of_char @rust_buf.len, value.bytes
    end
  end

  # Public so RustBufferBuilderMixin module functions can pack without
  # `include` flattening instance methods onto this class.
  # The class itself is private_constant. `reserve` stays private.
  def pack_into(size, format, value)
    reserve(size) do
      @rust_buf.data.put_array_of_char @rust_buf.len, [value].pack(format).bytes
    end
  end

  private

  def reserve(num_bytes)
    if @rust_buf.len + num_bytes > @rust_buf.capacity
      @rust_buf = RustBuffer.reserve(@rust_buf, num_bytes)
    end

    yield

    @rust_buf.len += num_bytes
  end
end

private_constant :RustBufferBuilder
