
{% import "macros.swift" as swift %}
{%- let obj = self.inner() %}
public protocol {{ obj.name() }}Protocol {
    {% for meth in obj.methods() -%}
    func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_protocol(meth) %}) {% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_swift -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

public class {{ obj.name()|class_name_swift }}: {{ obj.name() }}Protocol {
    fileprivate let pointer: UnsafeMutableRawPointer

    // TODO: We'd like this to be `private` but for Swifty reasons,
    // we can't implement `ViaFfi` without making this `required` and we can't
    // make it `required` without making it `public`.
    required init(unsafeFromRawPointer pointer: UnsafeMutableRawPointer) {
        self.pointer = pointer
    }

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    public convenience init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.init(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }
    {%- when None %}
    {%- endmatch %}

    deinit {
        try! rustCall { {{ obj.ffi_object_free().name() }}(pointer, $0) }
    }

    {% for cons in obj.alternate_constructors() %}
    public static func {{ cons.name()|fn_name_swift }}({% call swift::arg_list_decl(cons) %}) {% call swift::throws(cons) %} -> {{ obj.name()|class_name_swift }} {
        return {{ obj.name()|class_name_swift }}(unsafeFromRawPointer: {% call swift::to_ffi_call(cons) %})
    }
    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_swift }} {
        let _retval = {% call swift::to_ffi_call_with_prefix("self.pointer", meth) %}
        return {% call swift::try(meth) %} {{ "_retval"|lift_swift(return_type) }}
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.pointer", meth) %}
    }
    {%- endmatch %}
    {% endfor %}
}


fileprivate extension {{ obj.name()|class_name_swift }} {
    fileprivate typealias FfiType = UnsafeMutableRawPointer

    fileprivate static func read(from buf: Reader) throws -> Self {
        let v: UInt64 = try buf.readInt()
        // The Rust code won't compile if a pointer won't fit in a UInt64.
        // We have to go via `UInt` because that's the thing that's the size of a pointer.
        let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: v))
        if (ptr == nil) {
            throw UniffiInternalError.unexpectedNullPointer
        }
        return try self.lift(ptr!)
    }

    fileprivate func write(into buf: Writer) {
        // This fiddling is because `Int` is the thing that's the same size as a pointer.
        // The Rust code won't compile if a pointer won't fit in a `UInt64`.
        buf.writeInt(UInt64(bitPattern: Int64(Int(bitPattern: self.lower()))))
    }

    fileprivate static func lift(_ pointer: UnsafeMutableRawPointer) throws -> Self {
        return Self(unsafeFromRawPointer: pointer)
    }

    fileprivate func lower() -> UnsafeMutableRawPointer {
        return self.pointer
    }
}

// Ideally this would be `fileprivate`, but Swift says:
// """
// 'private' modifier cannot be used with extensions that declare protocol conformances
// """
extension {{ obj.name()|class_name_swift }} : ViaFfi, Serializable {}
