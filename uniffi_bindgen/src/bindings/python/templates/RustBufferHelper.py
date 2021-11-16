# Types conforming to `Primitive` pass themselves directly over the FFI.
class Primitive:
    @classmethod
    def _lift(cls, value):
        return value

    @classmethod
    def _lower(cls, value):
        return value

# Helper class for new types that will always go through a RustBuffer.
# Classes should inherit from this and implement the `_read` static method
# and `_write` instance methods.
class ViaFfiUsingByteBuffer:
    @classmethod
    def _lift(cls, rbuf):
        with rbuf.consumeWithStream() as stream:
            return cls._read(stream)

    def _lower(self):
        with RustBuffer.allocWithBuilder() as builder:
            self._write(builder)
            return builder.finalize()

# Helper class for wrapper types that will always go through a RustBuffer.
# Classes should inherit from this and implement the `_read` and `_write` static methods.
class FfiConverterUsingByteBuffer:
    @classmethod
    def _lift(cls, rbuf):
        with rbuf.consumeWithStream() as stream:
            return cls._read(stream)

    @classmethod
    def _lower(cls, value):
        with RustBuffer.allocWithBuilder() as builder:
            cls._write(value, builder)
            return builder.finalize()

# Helpers for structural types.

class FfiConverterSequence:
    @staticmethod
    def _write(value, buf, writeItem):
        items = len(value)
        buf.writeI32(items)
        for item in value:
            writeItem(item, buf)

    @staticmethod
    def _read(buf, readItem):
        count = buf.readI32()
        if count < 0:
            raise InternalError("Unexpected negative sequence length")

        items = []
        while count > 0:
            items.append(readItem(buf))
            count -= 1
        return items

class FfiConverterOptional:
    @staticmethod
    def _write(value, buf, writeItem):
        if value is None:
            buf.writeU8(0)
            return

        buf.writeU8(1)
        writeItem(value, buf)

    @staticmethod
    def _read(buf, readItem):
        flag = buf.readU8()
        if flag == 0:
            return None
        elif flag == 1:
            return readItem(buf)
        else:
            raise InternalError("Unexpected flag byte for optional type")

class FfiConverterDictionary:
    @staticmethod
    def _write(items, buf, writeItem):
        buf.writeI32(len(items))
        for (key, value) in items.items():
            writeItem(key, value, buf)

    @staticmethod
    def _read(buf, readItem):
        count = buf.readI32()
        if count < 0:
            raise InternalError("Unexpected negative map size")
        items = {}
        while count > 0:
            key, value = readItem(buf)
            items[key] = value
            count -= 1
        return items
