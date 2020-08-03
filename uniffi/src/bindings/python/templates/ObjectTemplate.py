class {{ obj.name()|class_name_py }}(object):
    # XXX TODO: support for multiple constructors...
    {%- for cons in obj.constructors() %}
    def __init__(self, {% call py::arg_list_decl(cons.arguments()) -%}):
        {%- call py::coerce_args_extra_indent(cons.arguments()) %}
        self._handle = {% call py::to_rs_call(cons.ffi_func()) %}
    {%- endfor %}

    def __del__(self):
        _UniFFILib.{{ obj.ffi_object_free().name() }}(handle)

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth.arguments()) %}):
        {%- call py::coerce_args_extra_indent(meth.arguments()) %}
        _retval = {% call py::to_rs_call_with_prefix("self._handle", meth.ffi_func()) %}
        return {{ "_retval"|lift_py(return_type) }}
    
    {%- when None -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth.arguments()) %}):
        {%- call py::coerce_args_extra_indent(meth.arguments()) %}
        {% call py::to_rs_call_with_prefix("self._handle", meth.ffi_func()) %}
    {% endmatch %}
    {% endfor %}
