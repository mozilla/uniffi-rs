# Types conforming to `FfiConverterPrimitive` pass themselves directly over the FFI.
class FfiConverterPrimitive:
    @classmethod
    def check(cls, value):
        pass

    @classmethod
    def lift(cls, value):
        return value

    @classmethod
    def lower(cls, value):
        cls.check(value)
        return value

class FfiConverterPrimitiveInt(FfiConverterPrimitive):
    @classmethod
    def check(cls, value):
        if not cls.VALUE_MIN <= value < cls.VALUE_MAX:
            raise ValueError("{} requires {} <= value < {}".format(cls.CLASS_NAME, cls.VALUE_MIN, cls.VALUE_MAX))
        super().check(value)

# Helper class for wrapper types that will always go through a RustBuffer.
# Classes should inherit from this and implement the `read` and `write` static methods.
class FfiConverterRustBuffer:
    @classmethod
    def lift(cls, rbuf):
        with rbuf.consumeWithStream() as stream:
            return cls.read(stream)

    @classmethod
    def lower(cls, value):
        with RustBuffer.allocWithBuilder() as builder:
            cls.write(value, builder)
            return builder.finalize()
