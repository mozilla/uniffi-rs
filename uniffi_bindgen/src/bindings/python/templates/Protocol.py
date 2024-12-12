{# misnamed - a generic "abstract base class". Used as both a protocol and an ABC for traits. #}
class {{ protocol.name }}({{ protocol.base_class }}):
    {{ protocol.docstring|docindent(4) -}}
    {%- for meth in protocol.methods %}
    {{ meth|def }} {{ meth.name }}({{ meth|arg_list }}):
        {{ meth.docstring|docindent(8) -}}
        raise NotImplementedError
    {%- else %}
    pass
    {%- endfor %}
