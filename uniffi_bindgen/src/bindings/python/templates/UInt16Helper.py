class {{ type_node.ffi_converter_name }}(_UniffiConverterPrimitiveInt):
    CLASS_NAME = "u16"
    VALUE_MIN = 0
    VALUE_MAX = 2**16

    @staticmethod
    def read(buf):
        return buf.read_u16()

    @staticmethod
    def write(value, buf):
        buf.write_u16(value)
