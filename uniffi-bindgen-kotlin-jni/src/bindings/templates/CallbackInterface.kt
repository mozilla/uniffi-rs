public interface {{ cbi.name }} {
    {%- for meth in cbi.methods %}
    {%- let callable = meth.callable %}
    {% if callable.is_async %}suspend {% endif %}fun {{ callable.name_kt() }}({{ callable.arg_list() }}): {{ callable.return_type_kt() }}
    {% endfor %}
}
