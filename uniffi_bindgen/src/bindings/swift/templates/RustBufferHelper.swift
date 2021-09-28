// Protocols for converters we'll implement in templates

fileprivate protocol FfiConverter {
    associatedtype SwiftType
    associatedtype FfiType

    static func lift(_ ffiValue: FfiType) throws -> SwiftType
    static func lower(_ value: SwiftType) -> FfiType

    static func read(from: Reader) throws -> SwiftType
    static func write(_ value: SwiftType, into: Writer)
}

fileprivate protocol FfiConverterUsingByteBuffer: FfiConverter where FfiType == RustBuffer {
    // Empty, because we want to declare some helper methods in the extension below.
}

extension FfiConverterUsingByteBuffer {
    static func lower(_ value: SwiftType) -> FfiType {
        let writer = Writer()
        Self.write(value, into: writer)
        return RustBuffer(bytes: writer.bytes)
    }

    static func lift(_ buf: FfiType) throws -> SwiftType {
        let reader = Reader(data: Data(rustBuffer: buf))
        let value = try Self.read(from: reader)
        if reader.hasRemaining() {
          throw UniffiInternalError.incompleteData
        }
        buf.deallocate()
        return value
    }
}

// Helpers for structural types. Note that because of canonical_names, it /should/ be impossible
// to make another `FfiConverterSequence` etc just using the UDL.
fileprivate enum FfiConverterSequence {
    static func write<T>(_ value: [T], into buf: Writer, writeItem: (T, Writer) -> Void) {
        let len = Int32(value.count)
        buf.writeInt(len)
        for item in value {
            writeItem(item, buf)
        }
    }

    static func read<T>(from buf: Reader, readItem: (Reader) throws -> T) throws -> [T] {
        let len: Int32 = try buf.readInt()
        var seq = [T]()
        seq.reserveCapacity(Int(len))
        for _ in 0 ..< len {
            seq.append(try readItem(buf))
        }
        return seq
    }
}

fileprivate enum FfiConverterOptional {
    static func write<T>(_ value: T?, into buf: Writer, writeItem: (T, Writer) -> Void) {
        guard let value = value else {
            buf.writeInt(Int8(0))
            return
        }
        buf.writeInt(Int8(1))
        writeItem(value, buf)
    }

    static func read<T>(from buf: Reader, readItem: (Reader) throws -> T) throws -> T? {
        switch try buf.readInt() as Int8 {
        case 0: return nil
        case 1: return try readItem(buf)
        default: throw UniffiInternalError.unexpectedOptionalTag
        }
    }
}

fileprivate enum FfiConverterDictionary {
    static func write<T>(_ value: [String: T], into buf: Writer, writeItem: (String, T, Writer) -> Void) {
        let len = Int32(value.count)
        buf.writeInt(len)
        for (key, value) in value {
            writeItem(key, value, buf)
        }
    }

    static func read<T>(from buf: Reader, readItem: (Reader) throws -> (String, T)) throws -> [String: T] {
        let len: Int32 = try buf.readInt()
        var dict = [String: T]()
        dict.reserveCapacity(Int(len))
        for _ in 0..<len {
            let (key, value) = try readItem(buf)
            dict[key] = value
        }
        return dict
    }
}
