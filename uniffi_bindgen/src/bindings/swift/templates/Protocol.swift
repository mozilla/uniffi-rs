{%- call swift::docstring_value(protocol_docstring, 0) %}
{%- if protocol_class_only %}
public protocol {{ protocol_name }}: AnyObject, Sendable {
{%- else %}
public protocol {{ protocol_name }}: Sendable {
{%- endif %}
    {% for meth in methods.iter() -%}
    {%- call swift::docstring(meth, 4) %}
    func {{ meth.name()|fn_name }}({% call swift::arg_list_protocol(meth) %}) {% call swift::is_async(meth) -%}{% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}
