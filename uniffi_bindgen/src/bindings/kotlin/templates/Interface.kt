{%- call kt::docstring_value(interface_docstring, 0) %}{% endcall %}
public interface {{ interface_name }} {
    {% for meth in methods.iter() -%}
    {%- call kt::docstring(meth, 4) %}{% endcall %}
    {% if meth.is_async() -%}suspend {% endif -%}
    fun {{ meth.name()|fn_name }}({% call kt::arg_list(meth, true) %}{% endcall %})
    {%- match meth.return_type() -%}
    {%- when Some(return_type) %}: {{ return_type|type_name(ci) -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
    companion object
}

