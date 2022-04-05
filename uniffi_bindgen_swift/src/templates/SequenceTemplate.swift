{%- import "macros.swift" as swift -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
fileprivate struct {{ outer_type|ffi_converter_name }}: FfiConverterRustBuffer {
    typealias SwiftType = {{ outer_type|type_name }}

    static func write(_ value: {{ outer_type|type_name }}, into buf: Writer) {
        let len = Int32(value.count)
        buf.writeInt(len)
        for item in value {
            {{ inner_type|write_fn }}(item, into: buf)
        }
    }

    static func read(from buf: Reader) throws -> {{ outer_type|type_name }} {
        let len: Int32 = try buf.readInt()
        var seq = {{ outer_type|type_name }}()
        seq.reserveCapacity(Int(len))
        for _ in 0 ..< len {
            seq.append(try {{ inner_type|read_fn }}(from: buf))
        }
        return seq
    }
}
