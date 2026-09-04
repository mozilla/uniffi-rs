class RustBuffer < FFI::Struct
  layout :capacity, :uint64,
         :len,      :uint64,
         :data,     :pointer

  def self.alloc(size)
    return ::{{ self.module_name() }}.rust_call(:{{ ci.ffi_rustbuffer_alloc().name() }}, size)
  end

  def self.reserve(rbuf, additional)
    return ::{{ self.module_name() }}.rust_call(:{{ ci.ffi_rustbuffer_reserve().name() }}, rbuf, additional)
  end

  def free
    ::{{ self.module_name() }}.rust_call(:{{ ci.ffi_rustbuffer_free().name() }}, self)
  end

  def capacity
    self[:capacity]
  end

  def len
    self[:len]
  end

  def len=(value)
    self[:len] = value
  end

  def data
    self[:data]
  end

  def to_s
    "RustBuffer(capacity=#{capacity}, len=#{len}, data=#{data.read_bytes len})"
  end

  # The allocated buffer will be automatically freed if an error occurs, ensuring that
  # we don't accidentally leak it.
  def self.allocWithBuilder
    builder = RustBufferBuilder.new

    begin
      yield builder
    rescue => e
      builder.discard
      raise e
    end
  end

  # The RustBuffer will be freed once the context-manager exits, ensuring that we don't
  # leak it even if an error occurs.
  def consumeWithStream
    stream = RustBufferStream.new self

    yield stream

    raise RuntimeError, 'junk data left in buffer after consuming' if stream.remaining != 0
  ensure
    free
  end

  {%- for typ in ci.iter_local_types() -%}
  {%- let canonical_type_name = self::canonical_name(typ) -%}
  {%- match typ -%}

  {% when Type::String -%}
  # The primitive String type.

  def self.alloc_from_{{ canonical_type_name }}(value)
    RustBuffer.allocWithBuilder do |builder|
      builder.write value.encode('utf-8')
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return stream.read(stream.remaining).force_encoding(Encoding::UTF_8)
    end
  end

  {% when Type::Bytes -%}
  # The primitive Bytes type.

  def self.alloc_from_{{ canonical_type_name }}(value)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, value)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Timestamp -%}
  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Duration -%}
  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Record { name: record_name, .. } -%}
  {%- let rec = ci.get_record_definition(record_name).unwrap() -%}
  # The Record type {{ record_name }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    {%- for field in rec.fields() %}
    {{ self.check_lower_rb("v.{}"|format(field.name()|var_name_rb), field.as_type().borrow())? }}
    {%- endfor %}
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Enum { name: enum_name, .. }  -%}
  {%- let e = ci.get_enum_definition(enum_name).unwrap() -%}
  # The Enum type {{ enum_name }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    {%- if !e.is_flat() %}
    {%- for variant in e.variants() %}
    if v.{{ variant.name()|var_name_rb }}?
      {%- for field in variant.fields() %}
      {%- if field.name().is_empty() %}
        {{ self.check_lower_rb("v.values[{}]"|format(loop.index0), field.as_type().borrow())? }}
      {%- else %}
        {{ self.check_lower_rb("v.{}"|format(field.name()|var_name_rb), field.as_type().borrow())? }}
      {%- endif %}
      {%- endfor %}
      return
    end
    {%- endfor %}
    {%- endif %}
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Optional { inner_type } -%}
  # The Optional<T> type for {{ self::canonical_name(inner_type) }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    if !v.nil?
      {{ self.check_lower_rb("v", inner_type.borrow())? }}
    end
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize()
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Sequence { inner_type } -%}
  # The Sequence<T> type for {{ self::canonical_name(inner_type) }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    v.each do |item|
      {{ self.check_lower_rb("item", inner_type.borrow())? }}
    end
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize()
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Set { inner_type } -%}
  # The Set<T> type for {{ self::canonical_name(inner_type) }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    v.each do |item|
      {{ self.check_lower_rb("item", inner_type.borrow())? }}
    end
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize()
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {% when Type::Map { key_type: k, value_type: v } %}
  # The Map<T> type for {{ canonical_type_name }}.

  def self.check_lower_{{ canonical_type_name }}(v)
    v.each do |k, v|
      {{ self.check_lower_rb("k", k.borrow())? }}
      {{ self.check_lower_rb("v", v.borrow())? }}
    end
  end

  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize
    end
  end

  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  {%- else -%}
  {#- No code emitted for types that don't lower into a RustBuffer -#}
  {%- endmatch -%}
  {%- endfor %}

  {%- for typ in ci.iter_external_types() -%}
  {%- let canonical_type_name = self::canonical_name(typ) -%}
  {%- match typ %}
  {%- when Type::Record { .. } | Type::Enum { .. } %}
  # External type bridge: allocates locally, delegates write through the
  # bridge that routes reserve through this shared library's allocator.
  def self.alloc_from_{{ canonical_type_name }}(v)
    RustBuffer.allocWithBuilder do |builder|
      {{ self.rust_buffer_write(typ)? }}(builder, v)
      return builder.finalize()
    end
  end

  # External type bridge: frees locally, delegates read to external stream.
  def consume_into_{{ canonical_type_name }}
    consumeWithStream do |stream|
      return {{ self.rust_buffer_read(typ)? }}(stream)
    end
  end

  # External type bridge: check_lower only validates the Ruby value; it never
  # allocs, reserves, or frees a RustBuffer. Delegating to the defining crate
  # is safe and does not break the local-allocator invariant of the bridges
  # above.
  def self.check_lower_{{ canonical_type_name }}(v)
    ::{{ self.external_type_module(typ.module_path().unwrap()) }}::RustBuffer.check_lower_{{ canonical_type_name }}(v)
  end
  {%- else %}
  {%- endmatch %}
  {%- endfor %}
end

module UniFFILib
  class ForeignBytes < FFI::Struct
    layout :len,      :int32,
           :data,     :pointer

    def len
      self[:len]
    end

    def data
      self[:data]
    end

    def to_s
      "ForeignBytes(len=#{len}, data=#{data.read_bytes(len)})"
    end
  end
end

private_constant :UniFFILib
