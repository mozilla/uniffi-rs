{%- for meth in methods %}
{% if meth.callable.is_async %}suspend {% endif %}fun {{ meth.callable.name_kt() }}({{ meth.callable.arg_list_kt() }}): {{ meth.callable.return_type_kt() }} {
    {%- let jni_method_name = meth.jni_method_name %}
    {%- let callable = meth.callable %}
    {% filter indent(4) %}{%- include "CallableBody.kt" %}{% endfilter %}
}
{%- endfor %}
