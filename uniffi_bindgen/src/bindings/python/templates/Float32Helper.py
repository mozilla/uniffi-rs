class FfiConverterFloat(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterFloat._lift(buf.readFloat())

    @staticmethod
    def _write(value, buf):
        buf.writeFloat(FfiConverterFloat._lower(value))
