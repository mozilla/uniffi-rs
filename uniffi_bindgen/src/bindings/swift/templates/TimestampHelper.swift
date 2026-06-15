#if swift(>=5.8)
@_documentation(visibility: private)
#endif
fileprivate struct FfiConverterTimestamp: FfiConverterRustBuffer {
    typealias SwiftType = Date

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> Date {
        let seconds: Int64 = try readInt(&buf)
        let nanoseconds: UInt32 = try readInt(&buf)
        // Build the Date from whole seconds first, then add the nanoseconds as a
        // separate TimeInterval.  Date stores CFAbsoluteTime (seconds since 2001),
        // so adding a small fraction to the ~7.8e8 base is twice as precise as
        // adding it to the ~1.76e9 Unix-epoch value, because the smaller magnitude
        // leaves more mantissa bits for the sub-second part.
        if seconds >= 0 {
            return Date(timeIntervalSince1970: Double(seconds))
                .addingTimeInterval(Double(nanoseconds) / 1.0e9)
        } else {
            return Date(timeIntervalSince1970: Double(seconds))
                .addingTimeInterval(-Double(nanoseconds) / 1.0e9)
        }
    }

    public static func write(_ value: Date, into buf: inout [UInt8]) {
        var delta = value.timeIntervalSince1970
        var sign: Int64 = 1
        if delta < 0 {
            // The nanoseconds portion of the epoch offset must always be
            // positive, to simplify the calculation we will use the absolute
            // value of the offset.
            sign = -1
            delta = -delta
        }
        if delta.rounded(.down) > Double(Int64.max) {
            fatalError("Timestamp overflow, exceeds max bounds supported by Uniffi")
        }
        let seconds = Int64(delta)
        let nanoseconds = UInt32((delta - Double(seconds)) * 1.0e9)
        writeInt(&buf, sign * seconds)
        writeInt(&buf, nanoseconds)
    }
}
