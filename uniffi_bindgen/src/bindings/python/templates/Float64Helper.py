class {{ ffi_converter_name }}(_UniffiConverterPrimitiveFloat):
    @staticmethod
    def read(buf):
        return buf.read_double()

    @staticmethod
    def write(value, buf):
        buf.write_double(value)
