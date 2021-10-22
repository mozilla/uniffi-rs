class FfiConverterUInt32(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterUInt32._lift(buf.readU32())

    @staticmethod
    def _write(value, buf):
        buf.writeU32(FfiConverterUInt32._lower(value))
