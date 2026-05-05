#if swift(>=5.8)
@_documentation(visibility: private)
#endif
fileprivate struct FfiConverterString: FfiConverter {
    typealias SwiftType = String
    typealias FfiType = RustBuffer

    public static func lift(_ value: RustBuffer) throws -> String {
        defer {
            value.deallocate()
        }
        if value.data == nil {
            return String()
        }
        let bytes = UnsafeBufferPointer<UInt8>(start: value.data!, count: Int(value.len))
        // Use Swift's native UTF-8 decoder; `String(bytes:encoding:.utf8)` goes
        // through Foundation's NSString and silently strips a leading U+FEFF BOM.
        // Invalid UTF-8 substitutes U+FFFD instead of trapping (unreachable
        // given Rust's `String` invariant).
        return String(decoding: bytes, as: UTF8.self)
    }

    public static func lower(_ value: String) -> RustBuffer {
        return value.utf8CString.withUnsafeBufferPointer { ptr in
            // The swift string gives us int8_t, we want uint8_t.
            ptr.withMemoryRebound(to: UInt8.self) { ptr in
                // The swift string gives us a trailing null byte, we don't want it.
                let buf = UnsafeBufferPointer(rebasing: ptr.prefix(upTo: ptr.count - 1))
                return RustBuffer.from(buf)
            }
        }
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> String {
        let len: Int32 = try readInt(&buf)
        // See `lift` above for why we avoid Foundation's NSString-backed decoder here.
        return String(decoding: try readBytes(&buf, count: Int(len)), as: UTF8.self)
    }

    public static func write(_ value: String, into buf: inout [UInt8]) {
        let len = Int32(value.utf8.count)
        writeInt(&buf, len)
        writeBytes(&buf, value.utf8)
    }
}
