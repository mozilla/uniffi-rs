class RustCallStatus < FFI::Struct
  layout :code,    :int8,
         :error_buf, RustBuffer

  def code
    self[:code]
  end

  def error_buf
    self[:error_buf]
  end

  def to_s
    "RustCallStatus(code=#{self[:code]})"
  end
end

# These match the values from the uniffi::rustcalls module
CALL_SUCCESS = 0
CALL_ERROR = 1
CALL_PANIC = 2
{%- for e in ci.enum_definitions() %}
{% if ci.is_name_used_as_error(e.name()) %}
{% if e.is_flat() %}
class {{ e.name()|class_name_rb }}
    {%- for variant in e.variants() %}
    {{ variant.name()|class_name_rb }} = Class.new StandardError
    {%- endfor %}
{% else %}
module {{ e.name()|class_name_rb }}
  {%- for variant in e.variants() %}
  class {{ variant.name()|class_name_rb }} < StandardError
    {%- let named_fields = variant.has_fields() && !variant.fields()[0].name().is_empty() %}
    {%- if named_fields %}
    def initialize({% for field in variant.fields() %}{{ field.name()|var_name_rb }}:{% if !loop.last %}, {% endif %}{% endfor %})
        {% for field in variant.fields() %}
        @{{ field.name()|var_name_rb }} = {{ field.name()|var_name_rb }}
        {% endfor %}
        super()
    end
    {% else %}
    def initialize({% for field in variant.fields() %}v{{ loop.index }}{% if !loop.last %}, {% endif %}{% endfor %})
        {% if variant.has_fields() %}
        @values = [{% for field in variant.fields() %}v{{ loop.index }}{% if !loop.last %}, {% endif %}{% endfor %}]
        {% endif %}
        super()
    end
    {% endif %}
    {%- if variant.has_fields() %}
    {%- if named_fields %}

    attr_reader {% for field in variant.fields() %}:{{ field.name()|var_name_rb }}{% if !loop.last %}, {% endif %}{% endfor %}
    {%- else %}

    attr_reader :values

    def [](index)
        @values[index]
    end
    {%- endif %}
    {% endif %}

    def to_s
      {%- if named_fields %}
        "#{self.class.name}({% for field in variant.fields() %}{{ field.name()|var_name_rb }}=#{@{{ field.name()|var_name_rb }}.inspect}{% if !loop.last %}, {% endif %}{% endfor %})"

      {%- else %}
      {%- if variant.has_fields() %}
        "#{self.class.name}(#{@values.inspect})"
      {%- else %}
        "#{self.class.name}()"
      {%- endif %}
      {%- endif %}

    end

    {% for variant in e.variants() %}
    def {{ variant.name()|var_name_rb }}?
      instance_of? {{ e.name()|class_name_rb }}::{{ variant.name()|class_name_rb }}
    end
    {% endfor %}
  end
  {%- endfor %}
{% endif %}
end
{% endif %}
{%- endfor %}

private_constant :CALL_SUCCESS, :CALL_ERROR, :CALL_PANIC, :RustCallStatus

def self.consume_buffer_into_error(reader, rust_buffer)
  rust_buffer.consumeWithStream do |stream|
    return reader.call(stream)
  end
end

# This crate's bindings error (panics, protocol mismatches, corrupt buffers).
# Mixin readers/writers raise this class, so a consumer lifting this crate's types
# should rescue {{ self.module_name() }}::InternalError — not their own
# crate's InternalError. Matches Python InternalError / Kotlin InternalException.
class InternalError < StandardError
end

def self.rust_call(fn_name, *args)
  rust_call_with_error(nil, fn_name, *args)
end

def self.rust_call_with_error(error_reader, fn_name, *args)
  status = RustCallStatus.new
  args << status

  result = UniFFILib.public_send(fn_name, *args)

  case status.code
  when CALL_SUCCESS
    result
  when CALL_ERROR
    if error_reader.nil?
      status.error_buf.free
      raise InternalError, "CALL_ERROR with no error_reader set"
    end
    raise consume_buffer_into_error(error_reader, status.error_buf)
  when CALL_PANIC
    if status.error_buf.len > 0
      raise InternalError, {{ self.lift_rb("status.error_buf", &Type::String)? }}
    else
      raise InternalError, "Rust panic"
    end
  else
    raise InternalError, "Unknown call status: #{status.code}"
  end
end

private_class_method :consume_buffer_into_error
