extension Int16: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Self {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}