{%- call swift::docstring_value(protocol_docstring, 0) %}{% endcall %}
public protocol {{ protocol_name }}: AnyObject, Sendable {
    {% for meth in methods.iter() -%}
    {%- call swift::docstring(meth, 4) %}{% endcall %}
    func {{ meth.name()|fn_name }}({% call swift::arg_list_protocol(meth) %}{% endcall %}) {% call swift::is_async(meth) -%}{% endcall %}{% call swift::throws(meth) %}{% endcall -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}
