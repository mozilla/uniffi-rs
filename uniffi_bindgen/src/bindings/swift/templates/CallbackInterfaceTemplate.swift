{%- let cbi = ci|get_callback_interface_definition(name) %}
{%- let callback_handler = format!("uniffiCallbackHandler{}", name) %}
{%- let callback_init = format!("uniffiCallbackInit{}", name) %}
{%- let methods = cbi.methods() %}
{%- let protocol_name = type_name.clone() %}
{%- let ffi_init_callback = cbi.ffi_init_callback() %}

{% include "Protocol.swift" %}
{% include "CallbackInterfaceImpl.swift" %}

// FfiConverter protocol for callback interfaces
fileprivate struct {{ ffi_converter_name }} {
    fileprivate static var slab = UniffiSlab<{{ type_name }}>()
}

extension {{ ffi_converter_name }} : FfiConverter {
    typealias SwiftType = {{ type_name }}
    typealias FfiType = Int64

    public static func lift(_ handle: Int64) throws -> SwiftType {
        return try! slab.get(handle: handle)
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType {
        let handle: Int64 = try readInt(&buf)
        return try lift(handle)
    }

    public static func lower(_ v: SwiftType) -> Int64 {
        return try! slab.insert(value: v)
    }

    public static func write(_ v: SwiftType, into buf: inout [UInt8]) {
        writeInt(&buf, lower(v))
    }
}
