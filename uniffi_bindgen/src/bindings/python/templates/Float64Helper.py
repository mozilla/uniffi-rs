class FfiConverterDouble(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterDouble._lift(buf.readDouble())

    @staticmethod
    def _write(value, buf):
        buf.writeDouble(FfiConverterDouble._lower(value))
