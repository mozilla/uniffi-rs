{# misnamed - a generic "abstract base class". Used as both a protocol and an ABC for traits. #}
class {{ protocol.name }}({{ protocol.base_classes|join(", ") }}):
    {{ protocol.docstring|docstring(4) -}}
    {%- for meth in protocol.methods.iter() %}
    {%- let callable = meth.callable %}
    def {{ meth.callable.name }}(self, {% include "CallableArgs.py" %}):
        {{ meth.docstring|docstring(8) -}}
        raise NotImplementedError
    {%- else %}
    pass
    {%- endfor %}
