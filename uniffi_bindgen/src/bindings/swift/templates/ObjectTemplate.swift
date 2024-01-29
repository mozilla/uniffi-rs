{%- let obj = ci|get_object_definition(name) %}
{%- let (protocol_name, impl_class_name) = obj|object_names %}
{%- let methods = obj.methods() %}
{%- let protocol_docstring = obj.docstring() %}

{% include "Protocol.swift" %}

{%- call swift::docstring(obj, 0) %}
public class {{ impl_class_name }}:
    {%- for tm in obj.uniffi_traits() %}
    {%-     match tm %}
    {%-         when UniffiTrait::Display { fmt } %}
    CustomStringConvertible,
    {%-         when UniffiTrait::Debug { fmt } %}
    CustomDebugStringConvertible,
    {%-         when UniffiTrait::Eq { eq, ne } %}
    Equatable,
    {%-         when UniffiTrait::Hash { hash } %}
    Hashable,
    {%-         else %}
    {%-    endmatch %}
    {%- endfor %}
    {{ protocol_name }} {
    fileprivate let pointer: UnsafeMutableRawPointer

    // TODO: We'd like this to be `private` but for Swifty reasons,
    // we can't implement `FfiConverter` without making this `required` and we can't
    // make it `required` without making it `public`.
    required init(unsafeFromRawPointer pointer: UnsafeMutableRawPointer) {
        self.pointer = pointer
    }

    public func uniffiClonePointer() -> UnsafeMutableRawPointer {
        return try! rustCall { {{ obj.ffi_object_clone().name() }}(self.pointer, $0) }
    }

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    {%- call swift::docstring(cons, 4) %}
    public convenience init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.init(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }
    {%- when None %}
    {%- endmatch %}

    deinit {
        try! rustCall { {{ obj.ffi_object_free().name() }}(pointer, $0) }
    }

    {% for cons in obj.alternate_constructors() %}
    {%- call swift::docstring(cons, 4) %}
    public static func {{ cons.name()|fn_name }}({% call swift::arg_list_decl(cons) %}) {% call swift::throws(cons) %} -> {{ impl_class_name }} {
        return {{ impl_class_name }}(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }

    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- if meth.is_async() %}
    {%- call swift::docstring(meth, 4) %}
    public func {{ meth.name()|fn_name }}({%- call swift::arg_list_decl(meth) -%}) async {% call swift::throws(meth) %}{% match meth.return_type() %}{% when Some with (return_type) %} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
        return {% call swift::try(meth) %} await uniffiRustCallAsync(
            rustFutureFunc: {
                {{ meth.ffi_func().name() }}(
                    self.uniffiClonePointer()
                    {%- for arg in meth.arguments() -%}
                    ,
                    {{ arg|lower_fn }}({{ arg.name()|var_name }})
                    {%- endfor %}
                )
            },
            pollFunc: {{ meth.ffi_rust_future_poll(ci) }},
            completeFunc: {{ meth.ffi_rust_future_complete(ci) }},
            freeFunc: {{ meth.ffi_rust_future_free(ci) }},
            {%- match meth.return_type() %}
            {%- when Some(return_type) %}
            liftFunc: {{ return_type|lift_fn }},
            {%- when None %}
            liftFunc: { $0 },
            {%- endmatch %}
            {%- match meth.throws_type() %}
            {%- when Some with (e) %}
            errorHandler: {{ e|ffi_converter_name }}.lift
            {%- else %}
            errorHandler: nil
            {% endmatch %}
        )
    }

    {% else -%}

    {%- match meth.return_type() -%}

    {%- when Some with (return_type) %}
    {%- call swift::docstring(meth, 4) %}
    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_name }} {
        return {% call swift::try(meth) %} {{ return_type|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.uniffiClonePointer()", meth) %}
        )
    }

    {%- when None %}
    {%- call swift::docstring(meth, 4) %}
    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.uniffiClonePointer()", meth) %}
    }

    {%- endmatch -%}
    {%- endif -%}
    {% endfor %}

    {%- for tm in obj.uniffi_traits() %}
    {%-     match tm %}
    {%-         when UniffiTrait::Display { fmt } %}
    public var description: String {
        return {% call swift::try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.uniffiClonePointer()", fmt) %}
        )
    }
    {%-         when UniffiTrait::Debug { fmt } %}
    public var debugDescription: String {
        return {% call swift::try(fmt) %} {{ fmt.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.uniffiClonePointer()", fmt) %}
        )
    }
    {%-         when UniffiTrait::Eq { eq, ne } %}
    public static func == (lhs: {{ impl_class_name }}, other: {{ impl_class_name }}) -> Bool {
        return {% call swift::try(eq) %} {{ eq.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("lhs.uniffiClonePointer()", eq) %}
        )
    }
    {%-         when UniffiTrait::Hash { hash } %}
    public func hash(into hasher: inout Hasher) {
        let val = {% call swift::try(hash) %} {{ hash.return_type().unwrap()|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.uniffiClonePointer()", hash) %}
        )
        hasher.combine(val)
    }
    {%-         else %}
    {%-    endmatch %}
    {%- endfor %}

}

{%- if obj.has_callback_interface() %}
{%- let callback_handler = format!("uniffiCallbackInterface{}", name) %}
{%- let callback_init = format!("uniffiCallbackInit{}", name) %}
{%- let vtable = obj.vtable().expect("trait interface should have a vtable") %}
{%- let vtable_methods = obj.vtable_methods() %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.swift" %}
{%- endif %}

public struct {{ ffi_converter_name }}: FfiConverter {
    {%- if obj.has_callback_interface() %}
    fileprivate static var handleMap = UniffiHandleMap<{{ type_name }}>()
    {%- endif %}

    typealias FfiType = UnsafeMutableRawPointer
    typealias SwiftType = {{ type_name }}

    public static func lift(_ pointer: UnsafeMutableRawPointer) throws -> {{ type_name }} {
        return {{ impl_class_name }}(unsafeFromRawPointer: pointer)
    }

    public static func lower(_ value: {{ type_name }}) -> UnsafeMutableRawPointer {
        {%- if obj.has_callback_interface() %}
        guard let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: handleMap.insert(obj: value))) else {
            fatalError("Cast to UnsafeMutableRawPointer failed")
        }
        return ptr
        {%- else %}
        return value.uniffiClonePointer()
        {%- endif %}
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        let v: UInt64 = try readInt(&buf)
        // The Rust code won't compile if a pointer won't fit in a UInt64.
        // We have to go via `UInt` because that's the thing that's the size of a pointer.
        let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: v))
        if (ptr == nil) {
            throw UniffiInternalError.unexpectedNullPointer
        }
        return try lift(ptr!)
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        // This fiddling is because `Int` is the thing that's the same size as a pointer.
        // The Rust code won't compile if a pointer won't fit in a `UInt64`.
        writeInt(&buf, UInt64(bitPattern: Int64(Int(bitPattern: lower(value)))))
    }
}

{#
We always write these public functions just in case the enum is used as
an external type by another crate.
#}
public func {{ ffi_converter_name }}_lift(_ pointer: UnsafeMutableRawPointer) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}.lift(pointer)
}

public func {{ ffi_converter_name }}_lower(_ value: {{ type_name }}) -> UnsafeMutableRawPointer {
    return {{ ffi_converter_name }}.lower(value)
}
