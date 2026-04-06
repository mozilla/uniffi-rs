public interface {{ cbi.name }} {
    {%- for meth in cbi.methods %}
    fun {{ meth.callable.name_kt() }}({{ meth.callable.arg_list() }}): {{ meth.callable.return_type_kt() }}
    {% endfor %}
}
