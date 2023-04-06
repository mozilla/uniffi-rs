fileprivate struct FfiConverterUInt32: FfiConverterPrimitive {
    typealias FfiType = UInt32
    typealias DartType = UInt32

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> UInt32 {
        return try lift(readInt(&buf))
    }

    public static func write(_ value: DartType, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
