extension Float: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Self {
        return try self.lift(buf.readFloat())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeFloat(self.lower())
    }
}