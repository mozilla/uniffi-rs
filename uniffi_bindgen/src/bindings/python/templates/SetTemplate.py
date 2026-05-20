class {{ set.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @classmethod
    def check_lower(cls, value):
        for item in value:
            {{ set.inner.ffi_converter_name }}.check_lower(item)

    @classmethod
    def write(cls, value, buf):
        buf.write_i32(len(value))
        for item in value:
            {{ set.inner.ffi_converter_name }}.write(item, buf)

    @classmethod
    def read(cls, buf):
        count = buf.read_i32()
        if count < 0:
            raise InternalError("Unexpected negative set size")

        return {
            {{ set.inner.ffi_converter_name }}.read(buf) for i in range(count)
        }
