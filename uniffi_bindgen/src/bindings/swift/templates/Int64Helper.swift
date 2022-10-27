fileprivate struct FfiConverterInt64: FfiConverterPrimitive {
    typealias FfiType = Int64
    typealias SwiftType = Int64

    static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> Int64 {
        return try lift(readInt(&buf))
    }

    static func write(_ value: Int64, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
