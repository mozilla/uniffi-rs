class FfiConverterUInt32(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not 0 <= value < 2**32:
            raise ValueError("u32 requires {} <= value < {}".format(0, 2**32))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readU32()

    @staticmethod
    def write(value, buf):
        buf.writeU32(value)
