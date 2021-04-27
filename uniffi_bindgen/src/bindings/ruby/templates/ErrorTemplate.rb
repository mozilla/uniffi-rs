class RustError < FFI::Struct
  layout :code,    :int32,
         :message, :string

  def code
    self[:code]
  end

  def to_s
    "RustError(code=#{self[:code]}, message=#{self[:message]})"
  end
end

{% for e in ci.iter_error_definitions() %}
class {{ e.name()|class_name_rb }}
  {%- for value in e.values() %}
  {{ value|class_name_rb }} = Class.new StandardError
  {%- endfor %}

  def self.raise_err(code, message)
    {%- for value in e.values() %}
    if code == {{ loop.index }}
      raise {{ value|class_name_rb }}, message
    end
    {% endfor %}
    raise 'Unknown error code'
  end
end
{% endfor %}

class InternalError < StandardError
  def self.raise_err(code, message)
    raise InternalError, message
  end
end

def self.rust_call_with_error(error_class, fn_name, *args)
  error = RustError.new
  args << error

  result = UniFFILib.public_send(fn_name, *args)

  if error.code != 0
    message = error.to_s

    error_class.raise_err(error.code, message)
  end

  result
end
