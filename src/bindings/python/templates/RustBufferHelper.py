# Helpers for lifting/lowering primitive data types from/to a bytebuffer.

class RustBufferStream(object):

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

    def _pack_into(self, size, format, value):
        if self.offset + size > self.rbuf.len:
            raise RuntimeError("write past end of rust buffer")
        # XXX TODO: I feel like I should be able to use `struct.pack_into` here but can't figure it out.
        for i, byte in enumerate(struct.pack(format, value)):
            self.rbuf.data[self.offset + i] = byte
        self.offset += size

    def getByte(self):
        return self._unpack_from(1, ">c")

    def putByte(self, v):
        self._pack_into(1, ">c", v)

    def getDouble(self):
        return self._unpack_from(8, ">d")

    def putDouble(self, v):
        self._pack_into(8, ">d", v)

    def getInt(self):
        return self._unpack_from(4, ">i")

    def putInt(self, v):
        self._pack_into(8, ">i", v)
        
def liftOptional(rbuf, liftFrom):
    return liftFromOptional(RustBufferStream(rbuf), liftFrom)

def liftFromOptional(buf, liftFrom):
    if buf.getByte() == b"\x00":
        return None
    return liftFrom(buf)
