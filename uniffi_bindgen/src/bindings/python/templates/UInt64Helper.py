class FfiConverterUInt64(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterUInt64._lift(buf.readU64())

    @staticmethod
    def _write(value, buf):
        buf.writeU64(FfiConverterUInt64._lower(value))
