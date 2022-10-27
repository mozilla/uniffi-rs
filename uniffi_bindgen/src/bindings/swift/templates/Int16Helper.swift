fileprivate struct FfiConverterInt16: FfiConverterPrimitive {
    typealias FfiType = Int16
    typealias SwiftType = Int16

    static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> Int16 {
        return try lift(readInt(&buf))
    }

    static func write(_ value: Int16, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
