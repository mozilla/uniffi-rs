#if swift(>=5.8)
@_documentation(visibility: private)
#endif
fileprivate struct {{ ffi_converter_name }}: FfiConverterRustBuffer {
    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        let len = Int32(value.count)
        writeInt(&buf, len)
        for item in value {
            {{ inner_type|write_fn }}(item, into: &buf)
        }
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let len: Int32 = try readInt(&buf)
        var set = {{ type_name }}()
        set.reserveCapacity(Int(len))
        for _ in 0..<len {
            let item = try {{ inner_type|read_fn }}(from: &buf)
            set.insert(item)
        }
        return set
    }
}
