class {{ obj.name()|class_name_py }}(object):
    # XXX TODO: support for multiple constructors...
    {%- for cons in obj.constructors() %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::coerce_args_extra_indent(cons) %}
        self._handle = {% call py::to_ffi_call(cons) %}
    {%- endfor %}

    def __del__(self):
        rust_call_with_error(
            InternalError,
            _UniFFILib.{{ obj.ffi_object_free().name() }},
            self._handle
        )

    {% for meth in obj.methods() -%}
    {%- if meth.is_static() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    @staticmethod
    def {{ meth.name()|fn_name_py }}({% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        _retval = {% call py::to_ffi_call(meth) %}
        return {{ "_retval"|lift_py(return_type) }}

    {%- when None -%}
    @staticmethod
    def {{ meth.name()|fn_name_py }}({% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        {% call py::to_ffi_call(meth) %}
    {% endmatch %}

    {%- else -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        _retval = {% call py::to_ffi_call_with_prefix("self._handle", meth) %}
        return {{ "_retval"|lift_py(return_type) }}

    {%- when None -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        {% call py::to_ffi_call_with_prefix("self._handle", meth) %}
    {% endmatch %}
    {% endif %}
    {% endfor %}
