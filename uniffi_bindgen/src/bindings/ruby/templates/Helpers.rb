def self.uniffi_in_range(i, type_name, min, max)
  raise TypeError, "no implicit conversion of #{i} into Integer" unless i.respond_to?(:to_int)
  i = i.to_int
  raise RangeError, "#{type_name} requires #{min} <= value < #{max}" unless (min <= i && i < max)
  i
end

def self.uniffi_utf8(v)
  raise TypeError, "no implicit conversion of #{v} into String" unless v.respond_to?(:to_str)
  v = v.to_str.encode(Encoding::UTF_8)
  raise Encoding::InvalidByteSequenceError, "not a valid UTF-8 encoded string" unless v.valid_encoding?
  v
end

def self.uniffi_bytes(v)
  raise TypeError, "no implicit conversion of #{v} into String" unless v.respond_to?(:to_str)
  v.to_str
end

# Callback return codes
UNIFFI_CALLBACK_SUCCESS = 0
UNIFFI_CALLBACK_ERROR = 1
UNIFFI_CALLBACK_UNEXPECTED_ERROR = 2

# Call a method on a callback interface object, catching and reporting errors
# to Rust via the call_status.
# If error_class is provided, known errors of that type are reported as UNIFFI_CALLBACK_ERROR;
# all other errors are reported as UNIFFI_CALLBACK_UNEXPECTED_ERROR.
def self.uniffi_trait_interface_call(call_status, make_call, write_return_value, error_class = nil, lower_error = nil)
  begin
    write_return_value.call make_call.call
  rescue StandardError => e
    buf = if !error_class.nil? && uniffi_is_error_type?(e, error_class)
      call_status[:code] = UNIFFI_CALLBACK_ERROR
      lower_error.call e
    else
      call_status[:code] = UNIFFI_CALLBACK_UNEXPECTED_ERROR
      {{ self.lower_rb("e.inspect", &Type::String)? }}
    end

    error_buf = call_status[:error_buf]
    error_buf[:capacity] = buf[:capacity]
    error_buf[:len] = buf[:len]
    error_buf[:data] = buf[:data]
  end
end

# Check if an exception is a variant of the given error type.
# Error types in Ruby are either modules (non-flat enums) or classes (flat enums),
# with variant classes as constant within them.
# error_class should be the Ruby Class or Module constant (e.g. MyError
# for local errors, ::OtherModule::SomeError for external errors).
def self.uniffi_is_error_type?(e, error_class)
  # Object-as-error: error_class is a class itself (e.g. MyError < StandardError)
  if error_class.is_a?(Class) && e.is_a?(error_class)
    return true
  end

  # Enum error: error_class is a module with class constants for each variant
  error_class.constants.any? do |c|
    klass = error_class.const_get c
    klass.is_a?(Class) && e.is_a?(klass)
  end
end
