fileprivate struct FfiConverterDouble: FfiConverterPrimitive {
    typealias FfiType = Double
    typealias SwiftType = Double

    static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> Double {
        return try lift(readDouble(&buf))
    }

    static func write(_ value: Double, into buf: inout [UInt8]) {
        writeDouble(&buf, lower(value))
    }
}
