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

// Types conforming to `Liftable` know how to deserialize ("lift") themselves
// from a byte buffer. This is equivalent to the `Liftable` trait on the Rust
// side.
protocol Liftable {
    static func lift(from: Reader) throws -> Self
}

// Types conforming to `Lowerable` know how to serialize ("lower") themselves
// into a byte buffer. Equivalent to the `Lowerable` trait on the Rust side.
protocol Lowerable {
    func lower(into: Writer)
}

extension String: Liftable, Lowerable {
    static func fromFFIValue(_ v: UnsafeMutablePointer<CChar>) throws -> Self {
        defer {
            {{ ci.ffi_string_free().name() }}(v)
        }
        return String(cString: v)
    }
    func toFFIValue() -> Self {
        self
    }

    static func lift(from buf: Reader) throws -> Self {
        let len: UInt32 = try buf.readInt()
        return String(bytes: try buf.readBytes(count: Int(len)), encoding: String.Encoding.utf8)!
    }

    func lower(into buf: Writer) {
        let len = UInt32(self.utf8.count)
        buf.writeInt(len)
        buf.writeBytes(self.utf8)
    }
}

// Types conforming to `Primitive` pass themselves directly over the FFI.
// Roughly equivalent to the `ViaFfi` implementations for primitives in Rust.
protocol Primitive {}

extension Primitive {
    static func fromFFIValue(_ v: Self) throws -> Self {
        return v
    }

    func toFFIValue() -> Self {
        return self
    }
}

// Types conforming to `Serializable` pass themselves over the FFI using byte
// buffers. Roughly equivalent to the `ViaFfiUsingByteBuffer` trait in Rust.
protocol Serializable: Liftable & Lowerable {}

extension Serializable {
    static func fromFFIValue(_ buf: RustBuffer) throws -> Self {
      let reader = Reader(data: Data(rustBuffer: buf))
      let value = try Self.lift(from: reader)
      if reader.hasRemaining() {
          throw InternalError.incompleteData
      }
      buf.deallocate()
      return value
    }

    func toFFIValue() -> RustBuffer {
      let writer = Writer()
      self.lower(into: writer)
      return RustBuffer(bytes: writer.bytes)
    }
}

// Implement our protocols for the built-in types that we use.

extension Bool: Liftable, Lowerable {
    static func lift(from buf: Reader) throws -> Bool {
        return try self.fromFFIValue(buf.readInt())
    }

    func lower(into buf: Writer) {
        buf.writeInt(self.toFFIValue())
    }

    static func fromFFIValue(_ v: UInt8) throws -> Bool {
        return v != 0
    }

    func toFFIValue() -> UInt8 {
        return self ? 1 : 0
    }
}

extension UInt8: Liftable, Lowerable, Primitive {
    static func lift(from buf: Reader) throws -> UInt8 {
        return try self.fromFFIValue(buf.readInt())
    }

    func lower(into buf: Writer) {
        buf.writeInt(self.toFFIValue())
    }
}

extension UInt32: Liftable, Lowerable, Primitive {
    static func lift(from buf: Reader) throws -> UInt32 {
        return try self.fromFFIValue(buf.readInt())
    }

    func lower(into buf: Writer) {
        buf.writeInt(self.toFFIValue())
    }
}

extension UInt64: Liftable, Lowerable, Primitive {
    static func lift(from buf: Reader) throws -> UInt64 {
        return try self.fromFFIValue(buf.readInt())
    }

    func lower(into buf: Writer) {
        buf.writeInt(self.toFFIValue())
    }
}

extension Float: Liftable, Lowerable, Primitive {
    static func lift(from buf: Reader) throws -> Float {
        return try self.fromFFIValue(buf.readFloat())
    }

    func lower(into buf: Writer) {
        buf.writeFloat(self.toFFIValue())
    }
}

extension Double: Liftable, Lowerable, Primitive {
    static func lift(from buf: Reader) throws -> Double {
        return try self.fromFFIValue(buf.readDouble())
    }

    func lower(into buf: Writer) {
        buf.writeDouble(self.toFFIValue())
    }
}

extension Optional: Liftable where Wrapped: Liftable {
    static func lift(from buf: Reader) throws -> Self {
        switch try buf.readInt() as UInt8 {
        case 0: return nil
        case 1: return try Wrapped.lift(from: buf)
        default: throw InternalError.unexpectedOptionalTag
        }
    }
}

extension Optional: Lowerable where Wrapped: Lowerable {
    func lower(into buf: Writer) {
        guard let value = self else {
            buf.writeInt(UInt8(0))
            return
        }
        buf.writeInt(UInt8(1))
        value.lower(into: buf)
    }
}

extension Optional: Serializable where Wrapped: Liftable & Lowerable {}

extension Array: Liftable where Element: Liftable {
    static func lift(from buf: Reader) throws -> Self {
        let len: UInt32 = try buf.readInt()
        var seq = [Element]()
        seq.reserveCapacity(Int(len))
        for _ in 1...len {
            seq.append(try Element.lift(from: buf))
        }
        return seq
    }
}

extension Array: Lowerable where Element: Lowerable {
    func lower(into buf: Writer) {
        let len = UInt32(self.count)
        buf.writeInt(len)
        for item in self {
            item.lower(into: buf)
        }
    }
}

extension Array: Serializable where Element: Liftable & Lowerable {}
