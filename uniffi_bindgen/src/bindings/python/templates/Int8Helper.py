class FfiConverterInt8(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterInt8._lift(buf.readI8())

    @staticmethod
    def _write(value, buf):
        buf.writeI8(FfiConverterInt8._lower(value))
