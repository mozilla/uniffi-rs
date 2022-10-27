fileprivate struct FfiConverterUInt8: FfiConverterPrimitive {
    typealias FfiType = UInt8
    typealias SwiftType = UInt8

    static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> UInt8 {
        return try lift(readInt(&buf))
    }

    static func write(_ value: UInt8, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
