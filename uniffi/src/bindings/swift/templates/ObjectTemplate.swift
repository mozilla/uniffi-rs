public class {{ obj.name() }} {
    private let handle: UInt64

    {%- for cons in obj.constructors() %}
    public init({% call swift::arg_list_decl(cons.arguments()) -%}) {
        self.handle = {% call swift::to_rs_call(cons) %}
    }
    {%- endfor %}

    // XXX TODO: destructors or equivalent.

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth.arguments()) %}) -> {{ return_type|decl_swift }} {
        let _retval = {% call swift::to_rs_call_with_prefix("self.handle", meth) %}
        return try! {{ "_retval"|lift_swift(return_type) }}
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth.arguments()) %}) {
        {% call swift::to_rs_call_with_prefix("self.handle", meth) %}
    }
    {%- endmatch %}
    {% endfor %}
}