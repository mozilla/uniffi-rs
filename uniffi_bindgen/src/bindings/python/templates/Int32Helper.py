class FfiConverterInt32(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not -2**31 <= value < 2**31:
            raise ValueError("i32 requires {} <= value < {}".format(-2**31, 2**31))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readI32()

    @staticmethod
    def write(value, buf):
        buf.writeI32(value)
