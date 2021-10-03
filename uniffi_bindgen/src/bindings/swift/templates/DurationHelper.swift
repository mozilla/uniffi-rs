extension TimeInterval: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let seconds: UInt64 = try buf.readInt()
        let nanoseconds: UInt32 = try buf.readInt()
        return Double(seconds) + (Double(nanoseconds) / 1.0e9)
    }

    fileprivate func write(into buf: Writer) {
        if self.rounded(.down) > Double(Int64.max) {
            fatalError("Duration overflow, exceeds max bounds supported by Uniffi")
        }

        if self < 0 {
            fatalError("Invalid duration, must be non-negative")
        }

        let seconds = UInt64(self)
        let nanoseconds = UInt32((self - Double(seconds)) * 1.0e9)
        buf.writeInt(seconds)
        buf.writeInt(nanoseconds)
    }
}