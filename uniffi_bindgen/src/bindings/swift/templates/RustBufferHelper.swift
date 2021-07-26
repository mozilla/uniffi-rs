// For every type used in the interface, we provide helper methods for conveniently
// lifting and lowering that type from C-compatible data, and for reading and writing
// values of that type in a buffer.

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
{% if ci.contains_optional_types() %}
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
{% endif %}

{% if ci.contains_sequence_types() %}
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
{% endif %}

{% if ci.contains_map_types() %}
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
{% endif %}

{% for typ in ci.iter_types() %}
{% let canonical_type_name = typ.canonical_name()|class_name_swift %}
{%- match typ -%}

{% when Type::String -%}
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

{% when Type::Boolean -%}
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

{% when Type::Timestamp -%}
extension Date: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Self {
        let seconds: Int64 = try buf.readInt()
        let nanoseconds: UInt32 = try buf.readInt()
        if seconds >= 0 {
            let delta = Double(seconds) + (Double(nanoseconds) / 1.0e9)
            return Date.init(timeIntervalSince1970: delta)
        } else {
            let delta = Double(seconds) - (Double(nanoseconds) / 1.0e9)
            return Date.init(timeIntervalSince1970: delta)
        }
    }

    fileprivate func write(into buf: Writer) {
        var delta = self.timeIntervalSince1970
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
        buf.writeInt(sign * seconds)
        buf.writeInt(nanoseconds)
    }
}

{% when Type::Duration -%}
extension TimeInterval {
    fileprivate static func liftDuration(_ buf: RustBuffer) throws -> Self {
      let reader = Reader(data: Data(rustBuffer: buf))
      let value = try Self.readDuration(from: reader)
      if reader.hasRemaining() {
          throw UniffiInternalError.incompleteData
      }
      buf.deallocate()
      return value
    }

    fileprivate func lowerDuration() -> RustBuffer {
      let writer = Writer()
      self.writeDuration(into: writer)
      return RustBuffer(bytes: writer.bytes)
    }

    fileprivate static func readDuration(from buf: Reader) throws -> Self {
        let seconds: UInt64 = try buf.readInt()
        let nanoseconds: UInt32 = try buf.readInt()
        return Double(seconds) + (Double(nanoseconds) / 1.0e9)
    }

    fileprivate func writeDuration(into buf: Writer) {
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

{% when Type::UInt8 -%}
extension UInt8: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt8 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::Int8 -%}
extension Int8: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int8 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::UInt16 -%}
extension UInt16: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt16 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::Int16 -%}
extension Int16: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int16 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::UInt32 -%}
extension UInt32: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt32 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::Int32 -%}
extension Int32: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int32 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::UInt64 -%}
extension UInt64: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> UInt64 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::Int64 -%}
extension Int64: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Int64 {
        return try self.lift(buf.readInt())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeInt(self.lower())
    }
}

{% when Type::Float32 -%}
extension Float: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Float {
        return try self.lift(buf.readFloat())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeFloat(self.lower())
    }
}

{% when Type::Float64 -%}
extension Double: Primitive, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> Double {
        return try self.lift(buf.readDouble())
    }

    fileprivate func write(into buf: Writer) {
        buf.writeDouble(self.lower())
    }
}

{% else %}
{# The methods for lifting/lowering/serializing this type are implemented inline with the type itself #}

{% endmatch %}
{% endfor %}
