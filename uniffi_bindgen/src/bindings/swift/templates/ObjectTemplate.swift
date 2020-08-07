public class {{ obj.name() }} {
    private let handle: UInt64

    {%- for cons in obj.constructors() %}
    public init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.handle = {% call swift::to_rs_call(cons) %}
    }
    {%- endfor %}

    deinit {
        {{ obj.ffi_object_free().name() }}(handle)
    }

    // TODO: Maybe merge the two templates (i.e the one with a return type and the one without)
    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|decl_swift }} {
        let _retval = {% call swift::to_rs_call_with_prefix("self.handle", meth) %}
        return {% call swift::try(meth) %} {{ "_retval"|lift_swift(return_type) }}
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_rs_call_with_prefix("self.handle", meth) %}
    }
    {%- endmatch %}
    {% endfor %}
}