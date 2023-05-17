class FfiConverterUInt16(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not 0 <= value < 2**16:
            raise ValueError("u16 requires {} <= value < {}".format(0, 2**16))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readU16()

    @staticmethod
    def write(value, buf):
        buf.writeU16(value)
