class FfiConverterInt16(Primitive):
    @staticmethod
    def _read(buf):
        return FfiConverterInt16._lift(buf.readI16())

    @staticmethod
    def _write(value, buf):
        buf.writeI16(FfiConverterInt16._lower(value))
