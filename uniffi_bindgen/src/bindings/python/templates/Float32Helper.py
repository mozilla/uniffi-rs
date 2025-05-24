class {{ type_node.ffi_converter_name }}(_UniffiConverterPrimitiveFloat):
    @staticmethod
    def read(buf):
        return buf.read_float()

    @staticmethod
    def write(value, buf):
        buf.write_float(value)
