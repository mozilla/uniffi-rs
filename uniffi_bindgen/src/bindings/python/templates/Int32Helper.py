class FfiConverterInt32(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterInt32._lift(buf.readI32())

    @staticmethod
    def _write(value, buf):
        buf.writeI32(FfiConverterInt32._lower(value))
