public interface {{ interface_name }} {% if is_interface_disposable == true -%} : Disposable {% endif -%} {
    {% for meth in methods.iter() -%}
    {% if meth.is_async() -%}suspend {% endif -%}
    fun {{ meth.name()|fn_name }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|protocol_name -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
    companion object
}
