fileprivate struct FfiConverterUInt16: FfiConverterPrimitive {
    typealias FfiType = UInt16
    typealias DartType = UInt16

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> UInt16 {
        return try lift(readInt(&buf))
    }

    public static func write(_ value: DartType, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
