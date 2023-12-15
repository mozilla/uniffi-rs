class UniffiConverterDouble(UniffiConverterPrimitiveFloat):
    @staticmethod
    def read(buf):
        return buf.read_double()

    @staticmethod
    def write(value, buf):
        buf.write_double(value)
