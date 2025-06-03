class {{ type_node.ffi_converter_name }}(_UniffiConverterPrimitiveInt):
    CLASS_NAME = "u64"
    VALUE_MIN = 0
    VALUE_MAX = 2**64

    @staticmethod
    def read(buf):
        return buf.read_u64()

    @staticmethod
    def write(value, buf):
        buf.write_u64(value)
