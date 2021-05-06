// Helper classes/extensions that don't change.
// Someday, this will be in a libray of its own.

fileprivate extension Data {
    init(rustBuffer: RustBuffer) {
        // TODO: This copies the buffer. Can we read directly from a
        // Rust buffer?
        self.init(bytes: rustBuffer.data!, count: Int(rustBuffer.len))
    }
}

// A helper class to read values out of a byte buffer.
fileprivate class Reader {
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
            throw UniffiInternalError.bufferOverflow
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
            throw UniffiInternalError.bufferOverflow
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
fileprivate class Writer {
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
        withUnsafeBytes(of: &value) { bytes.append(contentsOf: $0) }
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
fileprivate protocol Serializable {
    func write(into: Writer)
    static func read(from: Reader) throws -> Self
}

// Types confirming to `ViaFfi` can be transferred back-and-for over the FFI.
// This is analogous to the Rust trait of the same name.
fileprivate protocol ViaFfi: Serializable {
    associatedtype FfiType
    static func lift(_ v: FfiType) throws -> Self
    func lower() -> FfiType
}

// Types conforming to `Primitive` pass themselves directly over the FFI.
fileprivate protocol Primitive {}

extension Primitive {
    fileprivate typealias FfiType = Self

    fileprivate static func lift(_ v: Self) throws -> Self {
        return v
    }

    fileprivate func lower() -> Self {
        return self
    }
}

// Types conforming to `ViaFfiUsingByteBuffer` lift and lower into a bytebuffer.
// Use this for complex types where it's hard to write a custom lift/lower.
fileprivate protocol ViaFfiUsingByteBuffer: Serializable {}

extension ViaFfiUsingByteBuffer {
    fileprivate typealias FfiType = RustBuffer

    fileprivate static func lift(_ buf: RustBuffer) throws -> Self {
      let reader = Reader(data: Data(rustBuffer: buf))
      let value = try Self.read(from: reader)
      if reader.hasRemaining() {
          throw UniffiInternalError.incompleteData
      }
      buf.deallocate()
      return value
    }

    fileprivate func lower() -> RustBuffer {
      let writer = Writer()
      self.write(into: writer)
      return RustBuffer(bytes: writer.bytes)
    }
}

// Implement our protocols for the built-in types that we use.

extension String: ViaFfi {
    fileprivate typealias FfiType = RustBuffer

    fileprivate static func lift(_ v: FfiType) throws -> Self {
        defer {
            try! rustCall(UniffiInternalError.unknown("String.lift")) { err in
                {{ ci.ffi_rustbuffer_free().name() }}(v, err)
            }
        }
        if v.data == nil {
            return String()
        }
        let bytes = UnsafeBufferPointer<UInt8>(start: v.data!, count: Int(v.len))
        return String(bytes: bytes, encoding: String.Encoding.utf8)!
    }

    fileprivate func lower() -> FfiType {
        return self.utf8CString.withUnsafeBufferPointer { ptr in
            // The swift string gives us int8_t, we want uint8_t.
            ptr.withMemoryRebound(to: UInt8.self) { ptr in
                // The swift string gives us a trailing null byte, we don't want it.
                let buf = UnsafeBufferPointer(rebasing: ptr.prefix(upTo: ptr.count - 1))
                let bytes = ForeignBytes(bufferPointer: buf)
                return try! rustCall(UniffiInternalError.unknown("String.lower")) { err in
                    {{ ci.ffi_rustbuffer_from_bytes().name() }}(bytes, err)
                }
            }
        }
    }

    fileprivate static func read(from buf: Reader) throws -> Self {
        let len: Int32 = try buf.readInt()
        return String(bytes: try buf.readBytes(count: Int(len)), encoding: String.Encoding.utf8)!
    }

    fileprivate func write(into buf: Writer) {
        let len = Int32(self.utf8.count)
        buf.writeInt(len)
        buf.writeBytes(self.utf8)
    }
}

fileprivate enum JSONValue {
    fileprivate typealias JSONObject = Dictionary<String, Any>
    fileprivate typealias FfiType = RustBuffer
    
    fileprivate static func lift(_ v: FfiType) throws -> JSONObject {
        let jsonString = try String.lift(v)
        return try JSONValue.from(string: jsonString)
    }

    fileprivate static func read(from buf: Reader) throws -> JSONObject {
        let jsonString = try String.read(from: buf)
        return try JSONValue.from(string: jsonString)
    }

    fileprivate static func from(string: String) throws -> JSONObject {
        if let data = string.data(using: .utf8),
            let json = try JSONSerialization.jsonObject(with: data, options: []) as? [String: Any] {
                return json
            }
        throw UniffiInternalError.corruptData
    }

    fileprivate static func lower(_ jsonObject: JSONObject) -> FfiType {
        let jsonString = JSONValue.intoString(jsonObject)
        return jsonString.lower()
    }

    fileprivate static func write(_ jsonObject: JSONObject, into buf: Writer) {
        let jsonString = JSONValue.intoString(jsonObject)
        return jsonString.write(into: buf)
    }

    fileprivate static func intoString(_ jsonObject: JSONObject) -> String {
        if let data = try? JSONSerialization.data(withJSONObject: self, options: []),
            let string = String(data: data, encoding: .utf8) {
            return string
        }
        return "{}"
    }
}

extension Bool: ViaFfi {
    fileprivate typealias FfiType = Int8

    fileprivate static func read(from buf: Reader) throws -> Bool {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }

    fileprivate static func lift(_ v: Int8) throws -> Bool {
        return v != 0
    }

    fileprivate func lower() -> Int8 {
        return self ? 1 : 0
    }
}

extension UInt8: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt8 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int8: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int8 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt16: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt16 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int16: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int16 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt32: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt32 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int32: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int32 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension UInt64: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt64 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Int64: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int64 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

extension Float: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Float {
        return try self.lift(buf.readFloat())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeFloat(self.lower())
    }
}

extension Double: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Double {
        return try self.lift(buf.readDouble())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeDouble(self.lower())
    }
}

extension Optional: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Wrapped: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        switch try buf.readInt() as Int8 {
        case 0: return nil
        case 1: return try Wrapped.read(from: buf)
        default: throw UniffiInternalError.unexpectedOptionalTag
        }
    }

    fileprivate func write(into buf: Writer) {
        guard let value = self else {
            buf.writeInt(Int8(0))
            return
        }
        buf.writeInt(Int8(1))
        value.write(into: buf)
    }
}

extension Array: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Element: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let len: Int32 = try buf.readInt()
        var seq = [Element]()
        seq.reserveCapacity(Int(len))
        for _ in 0..<len {
            seq.append(try Element.read(from: buf))
        }
        return seq
    }

    fileprivate func write(into buf: Writer) {
        let len = Int32(self.count)
        buf.writeInt(len)
        for item in self {
            item.write(into: buf)
        }
    }
}

extension Dictionary: ViaFfiUsingByteBuffer, ViaFfi, Serializable where Key == String, Value: Serializable {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let len: Int32 = try buf.readInt()
        var dict = [String: Value]()
        dict.reserveCapacity(Int(len))
        for _ in 0..<len {
            dict[try String.read(from: buf)] = try Value.read(from: buf)
        }
        return dict
    }

    fileprivate func write(into buf: Writer) {
        let len = Int32(self.count)
        buf.writeInt(len)
        for (key, value) in self {
            key.write(into: buf)
            value.write(into: buf)
        }
    }
}
