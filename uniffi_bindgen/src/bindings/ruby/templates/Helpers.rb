def self.uniffi_in_range(i, type_name, min, max)
  i = i.to_i
  raise RangeError, "#{type_name} requires #{min} <= value < #{max}" unless (min <= i && i < max)
  i
end

def self.uniffi_utf8(v)
  v = v.to_s.encode(Encoding::UTF_8)
  raise Encoding::InvalidByteSequenceError, "not a valid UTF-8 encoded string" unless v.valid_encoding?
  v
end
