class FfiConverterUInt8(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterUInt8._lift(buf.readU8())

    @staticmethod
    def _write(value, buf):
        buf.writeU8(FfiConverterUInt8._lower(value))
