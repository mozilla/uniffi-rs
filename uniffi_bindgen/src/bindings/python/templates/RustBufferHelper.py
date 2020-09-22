# Helpers for lifting/lowering primitive data types from/to a bytebuffer.

class RustBufferStream(object):
    """Helper for structured reading of values for a RustBuffer."""

    def __init__(self, rbuf):
        self.rbuf = rbuf
        self.offset = 0

    @contextlib.contextmanager
    def checked_access(self, numBytes):
        if self.offset + numBytes > self.rbuf.len:
            raise RuntimeError("access past end of rust buffer")
        yield None
        self.offset += numBytes

    def _unpack_from(self, size, format):
        if self.offset + size > self.rbuf.len:
            raise RuntimeError("read past end of rust buffer")
        value = struct.unpack(format, self.rbuf.data[self.offset:self.offset+size])[0]
        self.offset += size
        return value

    def getByte(self):
        return self._unpack_from(1, ">c")

    def getDouble(self):
        return self._unpack_from(8, ">d")

    def getInt(self):
        return self._unpack_from(4, ">I")

    def getLong(self):
        return self._unpack_from(8, ">Q")

    def getString(self):
        numBytes = self.getInt()
        return self._unpack_from(numBytes, ">{}s".format(numBytes)).decode('utf-8')


class RustBufferBuilder(object):
    """Helper for structured writing of values into a RustBuffer."""

    def __init__(self):
        self.rbuf = RustBuffer.alloc(16)
        self.rbuf.len = 0

    def finalize(self):
        rbuf = self.rbuf
        self.rbuf = None
        return rbuf

    def discard(self):
        rbuf = self.finalize()
        rbuf.free()

    @contextlib.contextmanager
    def _reserve(self, numBytes):
        if self.rbuf.len + numBytes > self.rbuf.capacity:
            self.rbuf = RustBuffer.reserve(self.rbuf, numBytes)
        yield None
        self.rbuf.len += numBytes

    def _pack_into(self, size, format, value):
        with self._reserve(size):
            # XXX TODO: I feel like I should be able to use `struct.pack_into` here but can't figure it out.
            for i, byte in enumerate(struct.pack(format, value)):
                self.rbuf.data[self.rbuf.len + i] = byte

    def putByte(self, v):
        self._pack_into(1, ">c", v)

    def putDouble(self, v):
        self._pack_into(8, ">d", v)

    def putInt(self, v):
        self._pack_into(4, ">I", v)

    def putLong(self, v):
        self._pack_into(8, ">Q", v)

    def putString(self, v):
        valueBytes = v.encode('utf-8')
        numBytes = len(valueBytes)
        self.putInt(numBytes)
        self._pack_into(numBytes, ">{}s".format(numBytes), valueBytes)


def liftSequence(rbuf, liftFrom):
    return liftFromSequence(RustBufferStream(rbuf), liftFrom)

def liftFromSequence(buf, liftFrom):
    seq_len = buf.getInt()
    seq = []
    for i in range(0, seq_len):
        seq.append(listFrom(buf))
    return seq

def liftOptional(rbuf, liftFrom):
    return liftFromOptional(RustBufferStream(rbuf), liftFrom)

def liftFromOptional(buf, liftFrom):
    if buf.getByte() == b"\x00":
        return None
    return liftFrom(buf)

def liftString(cPtr):
    # TODO: update strings to lift from a `RustBuffer`.
    # There's currently no test coverage for this, so it can come in a separate PR
    # that cleans up a bunch of python lifting/lowering stuff.
    raise NotImplementedError
