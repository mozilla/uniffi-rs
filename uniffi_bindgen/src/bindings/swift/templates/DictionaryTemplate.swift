{%- import "macros.swift" as swift -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_swift %}
{%- let canonical_type_name = outer_type|canonical_name %}
fileprivate enum FfiConverter{{ canonical_type_name }}: FfiConverterUsingByteBuffer {
    typealias SwiftType = {{ outer_type|type_swift }}

    static func write(_ value: SwiftType, into buf: Writer) {
        let len = Int32(value.count)
        buf.writeInt(len)
        for (key, value) in value {
            {{ "key"|write_swift("buf", Type::String) }}
            {{ "value"|write_swift("buf", inner_type) }}
        }
    }

    static func read(from buf: Reader) throws -> SwiftType {
        let len: Int32 = try buf.readInt()
        var dict = SwiftType()
        dict.reserveCapacity(Int(len))
        for _ in 0..<len {
            dict[try {{ "buf"|read_swift(Type::String) }}] = try {{ "buf"|read_swift(inner_type) }}
        }
        return dict
    }
}