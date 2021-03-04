
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
    private let handle: UInt64

    private init(fromRawHandle handle: UInt64) {
        self.handle = handle
    }

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    public convenience init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.init(fromRawHandle: {% call swift::to_ffi_call(cons) %})
    }
    {%- when None %}
    {%- endmatch %}

    deinit {
        try! rustCall(UniffiInternalError.unknown("deinit")) { err in
            {{ obj.ffi_object_free().name() }}(handle, err)
        }
    }

    {% for cons in obj.alternate_constructors() %}
    public static func {{ cons.name()|fn_name_swift }}({% call swift::arg_list_decl(cons) %}) {% call swift::throws(cons) %} -> {{ obj.name()|class_name_swift }} {
        return {{ obj.name()|class_name_swift }}(fromRawHandle: {% call swift::to_ffi_call(cons) %})
    }
    {% endfor %}

    {# // TODO: Maybe merge the two templates (i.e the one with a return type and the one without) #}
    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_swift }} {
        let _retval = {% call swift::to_ffi_call_with_prefix("self.handle", meth) %}
        return {% call swift::try(meth) %} {{ "_retval"|lift_swift(return_type) }}
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.handle", meth) %}
    }
    {%- endmatch %}
    {% endfor %}
}