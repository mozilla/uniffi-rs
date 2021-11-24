{% import "macros.swift" as swift %}
{%- let func = self.inner() %}
{%- match func.return_type() -%}
{%- when Some with (return_type) %}

public func {{ func.name()|fn_name }}({%- call swift::arg_list_decl(func) -%}) {% call swift::throws(func) %} -> {{ return_type|type_name }} {
    let _retval = {% call swift::to_ffi_call(func) %}
    return {% call swift::try(func) %} {{ "_retval"|lift_var(return_type) }}
}

{% when None -%}

public func {{ func.name()|fn_name }}({% call swift::arg_list_decl(func) %}) {% call swift::throws(func) %} {
    {% call swift::to_ffi_call(func) %}
}
{% endmatch %}
