class FfiConverterUInt8(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not 0 <= value < 2**8:
            raise ValueError("u8 requires {} <= value < {}".format(0, 2**8))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readU8()

    @staticmethod
    def write(value, buf):
        buf.writeU8(value)
