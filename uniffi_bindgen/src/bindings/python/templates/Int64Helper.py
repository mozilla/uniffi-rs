class FfiConverterInt64(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterInt64._lift(buf.readI64())

    @staticmethod
    def _write(value, buf):
        buf.writeI64(FfiConverterInt64._lower(value))
