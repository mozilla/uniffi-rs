class FFIConverterPrimitive:
    size = NotImplemented
    pack_fmt = NotImplemented


    @staticmethod
    def lower(builder, value):
        builder._pack_into(self.size, self.pack_fmt, value)

    @staticmethod
    def lift(stream, value):
        return stream._unpack_from(self.size, self.pack_fmt, value)
