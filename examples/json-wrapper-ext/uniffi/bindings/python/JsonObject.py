import json

class FfiConverterTypeJsonObject:
    @staticmethod
    def lift(v):
        return json.loads(FfiConverterString.lift(v))

    @staticmethod
    def lower(v):
        return FfiConverterString.lower(json.dumps(v))

    @classmethod
    def read(cls, stream):
        return json.loads(FfiConverterString.read(stream))

    @classmethod
    def write(cls, builder, v):
        FfiConverterString.write(builder, json.dumps(v))
