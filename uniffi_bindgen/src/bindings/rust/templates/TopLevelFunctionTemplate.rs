{%- match func.return_type() -%}
{%- when Some with (return_type) %}

pub fn {{ func.name()|fn_name_rs }}({%- call rs::arg_list_decl(func.arguments()) -%}) -> {{ return_type|type_rs }} {
    todo!()
}

{% when None -%}

pub fn {{ func.name()|fn_name_rs }}({%- call rs::arg_list_decl(func.arguments()) -%}) {
    todo!()
}
{% endmatch %}
