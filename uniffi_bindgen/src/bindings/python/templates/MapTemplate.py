class {{ map.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @classmethod
    def check_lower(cls, items):
        for (key, value) in items.items():
            {{ map.key.ffi_converter_name }}.check_lower(key)
            {{ map.value.ffi_converter_name }}.check_lower(value)

    @classmethod
    def write(cls, items, buf):
        buf.write_i32(len(items))
        for (key, value) in items.items():
            {{ map.key.ffi_converter_name }}.write(key, buf)
            {{ map.value.ffi_converter_name }}.write(value, buf)

    @classmethod
    def read(cls, buf):
        count = buf.read_i32()
        if count < 0:
            raise InternalError("Unexpected negative map size")

        # It would be nice to use a dict comprehension,
        # but in Python 3.7 and before the evaluation order is not according to spec,
        # so we we're reading the value before the key.
        # This loop makes the order explicit: first reading the key, then the value.
        d = {}
        for i in range(count):
            key = {{ map.key.ffi_converter_name }}.read(buf)
            val = {{ map.value.ffi_converter_name }}.read(buf)
            d[key] = val
        return d
