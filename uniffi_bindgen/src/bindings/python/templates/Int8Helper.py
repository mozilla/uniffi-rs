class FfiConverterInt8(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not -2**7 <= value < 2**7:
            raise ValueError("i8 requires {} <= value < {}".format(-2**7, 2**7))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readI8()

    @staticmethod
    def write(value, buf):
        buf.writeI8(value)
