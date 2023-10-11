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
    fileprivate static var handleMap = UniFFICallbackHandleMap<{{ type_name }}>()
}

extension {{ ffi_converter_name }} : FfiConverter {
    typealias SwiftType = {{ type_name }}
    // We can use Handle as the FfiType because it's a typealias to UInt64
    typealias FfiType = UniFFICallbackHandle

    public static func lift(_ handle: UniFFICallbackHandle) throws -> SwiftType {
        guard let callback = handleMap.get(handle: handle) else {
            throw UniffiInternalError.unexpectedStaleHandle
        }
        return callback
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType {
        let handle: UniFFICallbackHandle = try readInt(&buf)
        return try lift(handle)
    }

    public static func lower(_ v: SwiftType) -> UniFFICallbackHandle {
        return handleMap.insert(obj: v)
    }

    public static func write(_ v: SwiftType, into buf: inout [UInt8]) {
        writeInt(&buf, lower(v))
    }
}
