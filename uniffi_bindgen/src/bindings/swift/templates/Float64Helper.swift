extension Double: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Self {
        return try self.lift(buf.readDouble())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeDouble(self.lower())
    }
}