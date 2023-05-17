class FfiConverterUInt64(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not 0 <= value < 2**64:
            raise ValueError("u64 requires {} <= value < {}".format(0, 2**64))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readU64()

    @staticmethod
    def write(value, buf):
        buf.writeU64(value)
