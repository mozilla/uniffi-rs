{%- import "macros.swift" as swift -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_swift %}
{%- let canonical_type_name = outer_type|canonical_name %}
fileprivate enum FfiConverter{{ canonical_type_name }}: FfiConverterUsingByteBuffer {
    typealias SwiftType = {{ outer_type|type_swift }}

    static func write(_ value: SwiftType, into buf: Writer) {
        guard let value = value else {
            buf.writeInt(Int8(0))
            return
        }
        buf.writeInt(Int8(1))
        {{ "value"|write_swift("buf", inner_type) }}
    }

    static func read(from buf: Reader) throws -> SwiftType {
        switch try buf.readInt() as Int8 {
        case 0: return nil
        case 1: return try {{ "buf"|read_swift(inner_type) }}
        default: throw UniffiInternalError.unexpectedOptionalTag
        }
    }
}