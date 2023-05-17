class FfiConverterInt16(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not -2**15 <= value < 2**15:
            raise ValueError("i16 requires {} <= value < {}".format(-2**15, 2**15))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readI16()

    @staticmethod
    def write(value, buf):
        buf.writeI16(value)
