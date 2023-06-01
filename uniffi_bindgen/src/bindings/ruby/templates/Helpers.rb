def uniffi_in_range(i, type_name, min, max)
  i = i.to_i
  raise RangeError, "#{type_name} requires #{min} <= value < #{max}" unless (min <= i && i < max)
  i
end
