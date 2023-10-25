{%- let obj = ci|get_object_definition(name) %}
{%- let (protocol_name, impl_class_name) = obj|object_names %}
{%- let methods = obj.methods() %}

{% include "Protocol.swift" %}

public class {{ impl_class_name }}: {{ protocol_name }} {
    fileprivate let handle: Int64

    // TODO: We'd like this to be `private` but for Swifty reasons,
    // we can't implement `FfiConverter` without making this `required` and we can't
    // make it `required` without making it `public`.
    required init(handle: Int64) {
        self.handle = handle
    }

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    public convenience init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.init(handle: {% call swift::to_ffi_call(cons) %})
    }
    {%- when None %}
    {%- endmatch %}

    deinit {
        try! rustCall { {{ obj.ffi_object_free().name() }}(handle, $0) }
    }

    internal func uniffiCloneHandle() -> Int64 {
        try! rustCall { {{ obj.ffi_object_inc_ref().name() }}(handle, $0) }
        return handle
    }

    {% for cons in obj.alternate_constructors() %}

    public static func {{ cons.name()|fn_name }}({% call swift::arg_list_decl(cons) %}) {% call swift::throws(cons) %} -> {{ impl_class_name }} {
        return {{ impl_class_name }}(handle: {% call swift::to_ffi_call(cons) %})
    }

    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- if meth.is_async() %}

    public func {{ meth.name()|fn_name }}({%- call swift::arg_list_decl(meth) -%}) async {% call swift::throws(meth) %}{% match meth.return_type() %}{% when Some with (return_type) %} -> {{ return_type|type_name }}{% when None %}{% endmatch %} {
        return {% call swift::try(meth) %} await uniffiRustCallAsync(
            rustFutureFunc: {
                {{ meth.ffi_func().name() }}(
                    self.uniffiCloneHandle()
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

    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_name }} {
        return {% call swift::try(meth) %} {{ return_type|lift_fn }}(
            {% call swift::to_ffi_call_with_prefix("self.uniffiCloneHandle()", meth) %}
        )
    }

    {%- when None %}

    public func {{ meth.name()|fn_name }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.uniffiCloneHandle()", meth) %}
    }

    {%- endmatch -%}
    {%- endif -%}
    {% endfor %}
}

{%- if obj.is_trait_interface() %}
{%- let callback_handler = format!("uniffiCallbackInterface{}", name) %}
{%- let callback_init = format!("uniffiCallbackInit{}", name) %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.swift" %}
{%- endif %}

public struct {{ ffi_converter_name }}: FfiConverter {
    {%- if obj.is_trait_interface() %}
    fileprivate static var handleMap = UniFFICallbackHandleMap<{{ type_name }}>()
    {%- endif %}

    typealias FfiType = Int64
    typealias SwiftType = {{ type_name }}

    public static func lift(_ handle: Int64) throws -> {{ type_name }} {
        return {{ impl_class_name }}(handle: handle)
    }

    public static func lower(_ value: {{ type_name }}) -> Int64 {
        {%- match obj.imp() %}
        {%- when ObjectImpl::Struct %}
        // inc-ref the current handle, then return the new reference.
        return value.uniffiCloneHandle()
        {%- when ObjectImpl::Trait %}
        guard let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: handleMap.insert(obj: value))) else {
            fatalError("Cast to UnsafeMutableRawPointer failed")
        }
        return ptr
        {%- endmatch %}
    }

    public static func read(from buf: inout (data: Data, offset: Data.Index)) throws -> {{ type_name }} {
        return try lift(try readInt(&buf))
    }

    public static func write(_ value: {{ type_name }}, into buf: inout [UInt8]) {
        writeInt(&buf, lower(value))
    }
}

{#
We always write these public functions just in case the enum is used as
an external type by another crate.
#}
public func {{ ffi_converter_name }}_lift(_ handle: Int64) throws -> {{ type_name }} {
    return try {{ ffi_converter_name }}.lift(handle)
}

public func {{ ffi_converter_name }}_lower(_ value: {{ type_name }}) -> Int64 {
    return {{ ffi_converter_name }}.lower(value)
}
