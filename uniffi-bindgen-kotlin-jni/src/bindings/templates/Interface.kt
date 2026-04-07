public interface {{ int.name_kt() }} {
    {%- for meth in int.methods %}
    {% if meth.callable.is_async%}suspend {% endif %}fun {{ meth.callable.name_kt() }}({{ meth.callable.arg_list() }}): {{ meth.callable.return_type_kt() }}
    {% endfor %}
}
