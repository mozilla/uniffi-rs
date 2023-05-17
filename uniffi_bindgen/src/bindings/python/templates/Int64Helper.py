class FfiConverterInt64(FfiConverterPrimitive):
    @classmethod
    def lower(cls, value):
        if not -2**63 <= value < 2**63:
            raise ValueError("i64 requires {} <= value < {}".format(-2**63, 2**63))
        return super().lower(value)

    @staticmethod
    def read(buf):
        return buf.readI64()

    @staticmethod
    def write(value, buf):
        buf.writeI64(value)
