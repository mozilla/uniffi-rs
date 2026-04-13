class {{ box_.self_type.ffi_converter_name }}(_UniffiConverterRustBuffer):
    @classmethod
    def check_lower(cls, value):
        {{ box_.inner.ffi_converter_name }}.check_lower(value)

    @classmethod
    def write(cls, value, buf):
        {{ box_.inner.ffi_converter_name }}.write(value, buf)

    @classmethod
    def read(cls, buf):
        return {{ box_.inner.ffi_converter_name }}.read(buf)
