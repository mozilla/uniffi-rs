{%- let obj = ci.get_object_definition(name).unwrap() %}
{%- let (protocol_name, impl_class_name) = obj|object_names %}
{%- let methods = obj.methods() %}
{%- let protocol_docstring = obj.docstring() %}

{%- let is_error = ci.is_name_used_as_error(name) %}

{% include "Protocol.swift" %}

{%- call swift::docstring(obj, 0) %}
open class {{ impl_class_name }}: {{ protocol_name }}, @unchecked Sendable {
    fileprivate let handle: UInt64

    /// Used to instantiate a [FFIObject] without an actual handle, for fakes in tests, mostly.
#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public struct NoHandle {
        public init() {}
    }

    // TODO: We'd like this to be `private` but for Swifty reasons,
    // we can't implement `FfiConverter` without making this `required` and we can't
    // make it `required` without making it `public`.
#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    required public init(unsafeFromHandle handle: UInt64) {
        self.handle = handle
    }

    // This constructor can be used to instantiate a fake object.
    // - Parameter noHandle: Placeholder value so we can have a constructor separate from the default empty one that may be implemented for classes extending [FFIObject].
    //
    // - Warning:
    //     Any object instantiated with this constructor cannot be passed to an actual Rust-backed object. Since there isn't a backing handle the FFI lower functions will crash.
#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public init(noHandle: NoHandle) {
        self.handle = 0
    }

#if swift(>=5.8)
    @_documentation(visibility: private)
#endif
    public func uniffiCloneHandle() -> UInt64 {
        return try! rustCall { {{ obj.ffi_object_clone().name() }}(self.handle, $0) }
    }

    {%- match obj.primary_constructor() %}
    {%- when Some(cons) %}
    {%- call swift::ctor_decl(cons, 4) %}
    {%- when None %}
    // No primary constructor declared for this class.
    {%- endmatch %}

    deinit {
        try! rustCall { {{ obj.ffi_object_free().name() }}(handle, $0) }
    }

    {% for cons in obj.alternate_constructors() %}
    {%- call swift::func_decl("public static func", cons, 4) %}
    {% endfor %}

    {% for meth in obj.methods() -%}
    {%- call swift::func_decl("open func", meth, 4) %}
    {% endfor %}

    {%- for tm in obj.uniffi_traits() %}
    {%-     match tm %}
    {%-         when UniffiTrait::Display { fmt } %}
    open var description: String {
        return {% call swift::is_try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call(fmt) %}
        )
    }
    {%-         when UniffiTrait::Debug { fmt } %}
    open var debugDescription: String {
        return {% call swift::is_try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call(fmt) %}
        )
    }
    {%-         when UniffiTrait::Eq { eq, ne } %}
    public static func == (self: {{ impl_class_name }}, other: {{ impl_class_name }}) -> Bool {
        return {% call swift::is_try(eq) %} {{ eq.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call(eq) %}
        )
    }
    {%-         when UniffiTrait::Hash { hash } %}
    open func hash(into hasher: inout Hasher) {
        let val = {% call swift::is_try(hash) %} {{ hash.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call(hash) %}
        )
        hasher.combine(val)
    }
    {%-         when UniffiTrait::Ord { cmp } %}
    public static func < (self: {{ impl_class_name }}, other: {{ impl_class_name }}) -> Bool {
        return {% call swift::is_try(cmp) %} {{ cmp.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call(cmp) %}
        ) < 0
    }
    {%-         else %}
    {%-    endmatch %}
    {%- endfor %}

}

{%- if !obj.has_callback_interface() %}
{# Simple case: the interface can only be implemented in Rust #}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}: FfiConverter {
    typealias FfiType = UInt64
    typealias SwiftType = {{ type_name }}

    public static func lift(_ handle: UInt64) throws -> {{ type_name }} {
        return {{ impl_class_name }}(unsafeFromHandle: handle)
    }

    public static func lower(_ value: {{ type_name }}) -> UInt64 {
        return value.uniffiCloneHandle()
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let handle: UInt64 = try readInt(&buf)
        return try lift(handle)
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}
{%- else %}
{# 
 # The interface can be implemented in Rust or Swift
 # * Generate a callback interface implementation to handle the Swift side
 # * In the FfiConverter, check which side a handle came from to know how to handle correctly.
#}
{%- let callback_handler = format!("uniffiCallbackInterface{}", name) %}
{%- let callback_init = format!("uniffiCallbackInit{}", name) %}
{%- let vtable = obj.vtable().expect("trait interface should have a vtable") %}
{%- let vtable_methods = obj.vtable_methods() %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.swift" %}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}: FfiConverter {
    fileprivate static let handleMap = UniffiHandleMap<{{ type_name }}>()

    typealias FfiType = UInt64
    typealias SwiftType = {{ type_name }}

    public static func lift(_ handle: UInt64) throws -> {{ type_name }} {
        if ((handle & 1) == 0) {
            // Rust-generated handle, construct a new class that uses the handle to implement the
            // interface
            return {{ impl_class_name }}(unsafeFromHandle: handle)
        } else {
            // Swift-generated handle, get the object from the handle map
            return try handleMap.remove(handle: handle)
        }
    }

    public static func lower(_ value: {{ type_name }}) -> UInt64 {
         if let rustImpl = value as? {{ impl_class_name }} {
             // Rust-implemented object.  Clone the handle and return it
            return rustImpl.uniffiCloneHandle()
         } else {
            // Swift object, generate a new vtable handle and return that.
            return handleMap.insert(obj: value)
         }
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let handle: UInt64 = try readInt(&buf)
        return try lift(handle)
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}

{%- endif %}

{%- for tm in obj.uniffi_traits() %}
{%-     match tm %}
{%-         when UniffiTrait::Display { .. } %}
extension {{ impl_class_name }}: CustomStringConvertible {}
{%-         when UniffiTrait::Debug { .. } %}
extension {{ impl_class_name }}: CustomDebugStringConvertible {}
{%-         when UniffiTrait::Eq { .. } %}
extension {{ impl_class_name }}: Equatable {}
{%-         when UniffiTrait::Hash { .. } %}
extension {{ impl_class_name }}: Hashable {}
{%-         when UniffiTrait::Ord { .. } %}
extension {{ impl_class_name }}: Comparable {}
{%-         else %}
{%-    endmatch %}
{%- endfor %}

{%- if is_error %}
extension {{ impl_class_name }}: Swift.Error {}
{% endif %}

{%- for t in obj.trait_impls() %}
extension {{impl_class_name}}: {{ self::trait_protocol_name(ci, t.trait_ty)? }} {}
{% endfor %}


{#
We always write these public functions just in case the object is used as
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
public func {{ ffi_converter_name }}_lower(_ value: {{ type_name }}) -> UInt64 {
    return {{ ffi_converter_name }}.lower(value)
}

{# Objects as error #}
{%- if is_error %}

{% if !config.omit_localized_error_conformance() %}
extension {{ type_name }}: Foundation.LocalizedError {
    public var errorDescription: String? {
        String(reflecting: self)
    }
}
{% endif %}

{# Due to some mismatches in the ffi converter mechanisms, errors are a RustBuffer storing a handle #}
#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public struct {{ ffi_converter_name }}__as_error: FfiConverterRustBuffer {
    public static func lift(_ buf: RustBuffer) throws -> {{ type_name }} {
        var reader = createReader(data: Data(rustBuffer: buf))
        return try {{ ffi_converter_name }}.read(from: &reader)
    }

    public static func lower(_ value: {{ type_name }}) -> RustBuffer {
        fatalError("not implemented")
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        fatalError("not implemented")
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        fatalError("not implemented")
    }
}

{# Error FFI converters also need these public functions. #}
#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}__as_error_lift(_ buf: RustBuffer) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}__as_error.lift(buf)
}

#if swift(>=5.8)
@_documentation(visibility: private)
#endif
public func {{ ffi_converter_name }}__as_error_lower(_ value: {{ type_name }}) -> RustBuffer {
    return {{ ffi_converter_name }}__as_error.lower(value)
}

{%- endif %}
