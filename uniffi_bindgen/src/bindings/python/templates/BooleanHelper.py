class _UniffiConverterBool:
    @classmethod
    def check(cls, value):
        pass

    @classmethod
    def lower(cls, value):
        return 1 if value else 0

    @staticmethod
    def lift(value):
        return value != 0

    @classmethod
    def read(cls, buf):
        return cls.lift(buf.read_u8())

    @classmethod
    def write(cls, value, buf):
        buf.write_u8(value)
