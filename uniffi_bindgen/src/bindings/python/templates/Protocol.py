{# misnamed - a generic "abstract base class". Used as both a protocol and an ABC for traits. #}
class {{ protocol_name }}({{ protocol_base_class }}):
    {%- call py::docstring_value(protocol_docstring, 4) %}
    {%- for meth in methods.iter() %}
    def {{ meth.name() }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::docstring(meth, 8) %}
        raise NotImplementedError
    {%- else %}
    pass
    {%- endfor %}
