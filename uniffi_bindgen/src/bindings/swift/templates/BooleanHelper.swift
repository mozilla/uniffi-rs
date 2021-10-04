extension Bool: ViaFfi {
    fileprivate typealias FfiType = Int8

    fileprivate static func read(from buf: Reader) throws -> Self {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }

    fileprivate static func lift(_ v: FfiType) throws -> Self {
        return v != 0
    }

    fileprivate func lower() -> FfiType {
        return self ? 1 : 0
    }
}