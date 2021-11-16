class FfiConverterBool:
    @staticmethod
    def _read(buf):
        return FfiConverterBool._lift(buf.readU8())

    @staticmethod
    def _write(value, buf):
        buf.writeU8(FfiConverterBool._lower(value))

    @staticmethod
    def _lift(value):
        return int(value) != 0

    @staticmethod
    def _lower(value):
        return 1 if value else 0
