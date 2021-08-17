from collections import namedtuple

PythonGuid = namedtuple('PythonGuid', "value")

class FfiConverterTypePythonGuid:
    @staticmethod
    def lift(v):
        return PythonGuid(FfiConverterString.lift(v))

    @staticmethod
    def lower(v):
        return FfiConverterString.lower(v.value)

    @classmethod
    def read(cls, stream):
        return PythonGuid(FfiConverterString.read(stream))

    @classmethod
    def write(cls, builder, v):
        FfiConverterString.write(builder, v.value)
