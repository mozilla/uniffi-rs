{%- let cbi = ci.get_callback_interface_definition(name).unwrap() %}
{%- let callback_handler = format!("uniffiCallbackHandler{}", name) %}
{%- let callback_init = format!("uniffiCallbackInit{}", name) %}
{%- let methods = cbi.methods() %}
{%- let protocol_name = type_name.clone() %}
{%- let protocol_docstring = cbi.docstring() %}
{%- let vtable = cbi.vtable() %}
{%- let vtable_methods = cbi.vtable_methods() %}
{%- let ffi_init_callback = cbi.ffi_init_callback() %}

{% include "Protocol.swift" %}
{% include "CallbackInterfaceImpl.swift" %}

// FfiConverter protocol for callback interfaces
#if swift(>=5.8)
@_documentation(visibility: private)
#endif
fileprivate struct {{ ffi_converter_name }} {
    fileprivate static let handleMap = UniffiHandleMap<{{ type_name }}>()
}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
extension {{ ffi_converter_name }} : FfiConverter {
    typealias SwiftType = {{ type_name }}
    typealias FfiType = UInt64

#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public static func lift(_ handle: UInt64) throws -> SwiftType {
        try handleMap.get(handle: handle)
    }

#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> SwiftType {
        let handle: UInt64 = try readInt(&buf)
        return try lift(handle)
    }

#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public static func lower(_ v: SwiftType) -> UInt64 {
        return handleMap.insert(obj: v)
    }

#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public static func write(_ v: SwiftType, into buf: inout [UInt8]) {
        writeInt(&buf, lower(v))
    }
}

{#
We always write these public functions just in case the callback is used as
an external type by another crate.
#}
#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}_lift(_ handle: UInt64) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}.lift(handle)
}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}_lower(_ v: {{ type_name }}) -> UInt64 {
    return {{ ffi_converter_name }}.lower(v)
}
