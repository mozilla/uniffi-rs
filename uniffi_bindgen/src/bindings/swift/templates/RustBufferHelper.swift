// Helper classes/extensions that don't change.
// Someday, this will be in a libray of its own.

// Serialization and deserialization errors.
enum InternalError: Error {
    // Reading the requested value would read past the end of the buffer.
    case bufferOverflow
    // The buffer still has data after lifting its containing value.
    case incompleteData
    // Unexpected tag byte for `Optional`; should be 0 or 1.
    case unexpectedOptionalTag
    // Unexpected integer that doesn't correspond to an enum case.
    case unexpectedEnumCase
    // Empty Result returned across the FFI
    case emptyResult
}

extension Data {
    init(rustBuffer: RustBuffer) {
        // TODO: This copies the buffer. Can we read directly from a
        // Rust buffer?
        self.init(bytes: rustBuffer.data!, count: Int(rustBuffer.len))
    }
}

// A helper class to read values out of a byte buffer.
class Reader {
    let data: Data
    var offset: Data.Index

    init(data: Data) {
        self.data = data
        self.offset = 0
    }

    // Reads an integer at the current offset, in big-endian order, and advances
    // the offset on success. Throws if reading the integer would move the
    // offset past the end of the buffer.
    func readInt<T: FixedWidthInteger>() throws -> T {
        let range = offset..<offset + MemoryLayout<T>.size
        guard data.count >= range.upperBound else {
            throw InternalError.bufferOverflow
        }
        if T.self == UInt8.self {
            let value = data[offset]
            offset += 1
            return value as! T
        }
        var value: T = 0
        let _ = withUnsafeMutableBytes(of: &value, { data.copyBytes(to: $0, from: range)})
        offset = range.upperBound
        return value.bigEndian
    }
    
    // Reads an arbitrary number of bytes, to be used to read
    // raw bytes, this is useful when lifting strings
    func readBytes(count: Int) throws -> Array<UInt8> {
        let range = offset..<(offset+count)
        guard data.count >= range.upperBound else {
            throw InternalError.bufferOverflow
        }
        var value = [UInt8](repeating: 0, count: count)
        value.withUnsafeMutableBufferPointer({ buffer in
            data.copyBytes(to: buffer, from: range)
        })
        offset = range.upperBound
        return value
    }

    // Reads a float at the current offset.
    @inlinable
    func readFloat() throws -> Float {
        return Float(bitPattern: try readInt())
    }

    // Reads a float at the current offset.
    @inlinable
    func readDouble() throws -> Double {
        return Double(bitPattern: try readInt())
    }

    // Indicates if the offset has reached the end of the buffer.
    @inlinable
    func hasRemaining() -> Bool {
        return offset < data.count
    }
}

// A helper class to write values into a byte buffer.
class Writer {
    var bytes: [UInt8]
    var offset: Array<UInt8>.Index

    init() {
        self.bytes = []
        self.offset = 0
    }

    func writeBytes<S>(_ byteArr: S) where S: Sequence, S.Element == UInt8 {
        bytes.append(contentsOf: byteArr)
    }

    // Writes an integer in big-endian order.
    //
    // Warning: make sure what you are trying to write
    // is in the correct type!
    func writeInt<T: FixedWidthInteger>(_ value: T) {
        var value = value.bigEndian
        let _ = withUnsafeBytes(of: &value, { bytes.append(contentsOf: $0) })
    }

    @inlinable
    func writeFloat(_ value: Float) {
        writeInt(value.bitPattern)
    }

    @inlinable
    func writeDouble(_ value: Double) {
        writeInt(value.bitPattern)
    }
}


// Types conforming to `Serializable` can be read and written in a bytebuffer.
protocol Serializable {
    func write(into: Writer)
    static func read(from: Reader) throws -> Self
}

// Types confirming to `ViaFfi` can be transferred back-and-for over the FFI.
// This is analogous to the Rust trait of the same name.
protocol ViaFfi: Serializable {
    associatedtype Value
    static func lift(_ v: Value) throws -> Self
    func lower() -> Value
}

// Types conforming to `Primitive` pass themselves directly over the FFI.
protocol Primitive {}

extension Primitive {
    typealias Value = Self

    static func lift(_ v: Self) throws -> Self {
        return v
    }

    func lower() -> Self {
        return self
    }
}

// Types conforming to `ViaFfiUsingByteBuffer` lift and lower into a bytebuffer.
// Use this for complex types where it's hard to write a custom lift/lower.
protocol ViaFfiUsingByteBuffer: Serializable {}

extension ViaFfiUsingByteBuffer {
    typealias Value = RustBuffer

    static func lift(_ buf: RustBuffer) throws -> Self {
      let reader = Reader(data: Data(rustBuffer: buf))
      let value = try Self.read(from: reader)
      if reader.hasRemaining() {
          throw InternalError.incompleteData
      }
      buf.deallocate()
      return value
    }

    func lower() -> RustBuffer {
      let writer = Writer()
      self.write(into: writer)
      return RustBuffer(bytes: writer.bytes)
    }
}

// Implement our protocols for the built-in types that we use.

extension String: ViaFfi {
    typealias Value = UnsafeMutablePointer<CChar>

    static func lift(_ v: Value) throws -> Self {
        defer {
            {{ ci.ffi_string_free().name() }}(v)
        }
        return String(cString: v)
    }

    func lower() -> Value {
        var rustErr = NativeRustError(code: 0, message: nil)
        let rustStr = {{ ci.ffi_string_alloc_from().name() }}(self, &rustErr)
        if rustErr.code != 0 {
            fatalError("caught a panic while passing a string across the ffi")
        }
        return rustStr
    }

    static func read(from buf: Reader) throws -> Self {
        let len: UInt32 = try buf.readInt()
        return String(bytes: try buf.readBytes(count: Int(len)), encoding: String.Encoding.utf8)!
    }

    func write(into buf: Writer) {
        let len = UInt32(self.utf8.count)
        buf.writeInt(len)
        buf.writeBytes(self.utf8)
    }
}


extension Bool: ViaFfi {
    typealias Value = UInt8

    static func read(from buf: Reader) throws -> Bool {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }

    static func lift(_ v: UInt8) throws -> Bool {
        return v != 0
    }

    func lower() -> UInt8 {
        return self ? 1 : 0
    }
}

extension UInt8: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> UInt8 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int8: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Int8 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt16: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> UInt16 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int16: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Int16 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt32: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> UInt32 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int32: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Int32 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt64: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> UInt64 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int64: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Int64 {
        return try self.lift(buf.readInt())
    }

    func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Float: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Float {
        return try self.lift(buf.readFloat())
    }

    func write(into buf: Writer) {
        buf.writeFloat(self.lower())
    }
}

extension Double: Primitive, ViaFfi {
    static func read(from buf: Reader) throws -> Double {
        return try self.lift(buf.readDouble())
    }

    func write(into buf: Writer) {
        buf.writeDouble(self.lower())
    }
}

extension Optional: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Wrapped: Serializable {
    static func read(from buf: Reader) throws -> Self {
        switch try buf.readInt() as UInt8 {
        case 0: return nil
        case 1: return try Wrapped.read(from: buf)
        default: throw InternalError.unexpectedOptionalTag
        }
    }

    func write(into buf: Writer) {
        guard let value = self else {
            buf.writeInt(UInt8(0))
            return
        }
        buf.writeInt(UInt8(1))
        value.write(into: buf)
    }
}

extension Array: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Element: Serializable {
    static func read(from buf: Reader) throws -> Self {
        let len: UInt32 = try buf.readInt()
        var seq = [Element]()
        seq.reserveCapacity(Int(len))
        for _ in 1...len {
            seq.append(try Element.read(from: buf))
        }
        return seq
    }

    func write(into buf: Writer) {
        let len = UInt32(self.count)
        buf.writeInt(len)
        for item in self {
            item.write(into: buf)
        }
    }
}
