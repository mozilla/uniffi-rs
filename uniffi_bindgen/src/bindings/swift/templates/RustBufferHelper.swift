// Implement our protocols for the built-in types that we use.
extension Optional: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Wrapped: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        switch try buf.readInt() as Int8 {
        case 0: return nil
        case 1: return try Wrapped.read(from: buf)
        default: throw UniffiInternalError.unexpectedOptionalTag
        }
    }

    fileprivate func write(into buf: Writer) {
        guard let value = self else {
            buf.writeInt(Int8(0))
            return
        }
        buf.writeInt(Int8(1))
        value.write(into: buf)
    }
}

extension Array: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Element: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let len: Int32 = try buf.readInt()
        var seq = [Element]()
        seq.reserveCapacity(Int(len))
        for _ in 0..<len {
            seq.append(try Element.read(from: buf))
        }
        return seq
    }

    fileprivate func write(into buf: Writer) {
        let len = Int32(self.count)
        buf.writeInt(len)
        for item in self {
            item.write(into: buf)
        }
    }
}

extension Dictionary: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Key == String, Value: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let len: Int32 = try buf.readInt()
        var dict = [String: Value]()
        dict.reserveCapacity(Int(len))
        for _ in 0..<len {
            dict[try String.read(from: buf)] = try Value.read(from: buf)
        }
        return dict
    }

    fileprivate func write(into buf: Writer) {
        let len = Int32(self.count)
        buf.writeInt(len)
        for (key, value) in self {
            key.write(into: buf)
            value.write(into: buf)
        }
    }
}
