public fun {{ func.callable.name_kt() }}({{ func.callable.arg_list() }}): {{ func.callable.return_type_kt() }} {
    {%- let jni_method_name = func.jni_method_name %}
    {%- let callable = func.callable %}
    {% filter indent(4) %}{%- include "CallableBody.kt" %}{% endfilter %}
}
