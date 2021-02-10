class {{ obj.name()|class_name_py }}(object):
    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::coerce_args_extra_indent(cons) %}
        self._handle = {% call py::to_ffi_call(cons) %}\
    {%- when None %}
    {%- endmatch %}

    def __del__(self):
        rust_call_with_error(
            InternalError,
            _UniFFILib.{{ obj.ffi_object_free().name() }},
            self._handle
        )

    {% for cons in obj.alternate_constructors() -%}
    @classmethod
    def {{ cons.name()|fn_name_py }}(cls, {% call py::arg_list_decl(cons) %}):
        {%- call py::coerce_args_extra_indent(cons) %}
        # Call the (fallible) function before creating any half-baked object instances.
        handle = {% call py::to_ffi_call(cons) %}
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required handle.
        inst = cls.__new__(cls)
        inst._handle = handle
        return inst
    {% endfor %}

    {% for meth in obj.methods() -%}
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
    {% endfor %}
