{%- import "macros.swift" as swift -%}
{%- let inner_type = self.inner() %}
{%- let outer_type = self.outer() %}
{%- let inner_type_name = inner_type|type_name %}
{%- let canonical_type_name = outer_type|canonical_name %}
fileprivate enum FfiConverter{{ canonical_type_name }}: FfiConverterUsingByteBuffer {
    typealias SwiftType = {{ outer_type|type_name }}

    static func write(_ value: SwiftType, into buf: Writer) {
        FfiConverterSequence.write(value, into: buf) { (item, buf) in
            {{ "item"|write_var("buf", inner_type) }}
        }
    }

    static func read(from buf: Reader) throws -> SwiftType {
        try FfiConverterSequence.read(from: buf) { buf in
            try {{ "buf"|read_var(inner_type) }}
        }
    }
}
