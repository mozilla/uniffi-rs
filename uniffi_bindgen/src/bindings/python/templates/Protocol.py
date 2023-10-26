class {{ protocol_name }}(typing.Protocol):
    {%- for meth in methods.iter() %}
    def {{ meth.name()|fn_name }}(self, {% call py::arg_list_decl(meth) %}):
        raise NotImplementedError
    {%- else %}
    pass
    {%- endfor %}
