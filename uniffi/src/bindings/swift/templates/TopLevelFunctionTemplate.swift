{%- match func.return_type() -%}
{%- when Some with (return_type) %}

public func {{ func.name()|fn_name_swift }}({%- call swift::arg_list_decl(func) -%}) {% call swift::throws(func) %} -> {{ return_type|decl_swift }} {
    let _retval = {% call swift::to_rs_call(func) %}
    return {%- call swift::try(func) %} {{ "_retval"|lift_swift(return_type) }}
}

{% when None -%}

public func {{ func.name()|fn_name_swift }}({% call swift::arg_list_decl(func) %}) {% call swift::throws(func) %} {
    {% call swift::to_rs_call(func) %}
}
{% endmatch %}