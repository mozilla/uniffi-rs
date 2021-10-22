class FfiConverterUInt16(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterUInt16._lift(buf.readU16())

    @staticmethod
    def _write(value, buf):
        buf.writeU16(FfiConverterUInt16._lower(value))
