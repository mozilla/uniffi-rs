from collections import namedtuple

Guid = namedtuple('Guid', "value")

class FfiConverterTypeGuid:
    @staticmethod
    def lift(v):
        return Guid(FfiConverterString.lift(v))

    @staticmethod
    def lower(v):
        return FfiConverterString.lower(v.value)

    @classmethod
    def read(cls, stream):
        return Guid(FfiConverterString.read(stream))

    @classmethod
    def write(cls, builder, v):
        FfiConverterString.write(builder, v.value)
