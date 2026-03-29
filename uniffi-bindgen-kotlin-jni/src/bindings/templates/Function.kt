public fun {{ func.callable.name_kt() }}({{ func.callable.arg_list_kt() }}): {{ func.callable.return_type_kt() }} {
    {%- let callable = func.callable %}
    {%- let jni_method_name = func.jni_method_name %}
    {%- filter indent(4) %}{% include "CallableBody.kt" %}{% endfilter %}
}
