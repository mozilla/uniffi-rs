class {{ obj.name()|class_name_py }}(object):
    # XXX TODO: support for multiple constructors...
    {%- for cons in obj.constructors() %}
    def __init__(self, {% for arg in cons.arguments() %}{{ arg.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
        {%- for arg in cons.arguments() %}
        {{ arg.name()|coerce_py(arg.type_()) }}
        {%- endfor %}
        self._handle = _UniFFILib.{{ cons.ffi_func().name() }}(
            {%- for arg in cons.arguments() %}
            {{ arg.name()|lower_py(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    {%- endfor %}

    # XXX TODO: destructors or equivalent.

    {%- for meth in obj.methods() %}
    def {{ meth.name()|fn_name_py }}(self, {% for arg in meth.arguments() %}{{ arg.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
        {%- for arg in meth.arguments() %}
        {{ arg.name()|coerce_py(arg.type_()) }}
        {%- endfor %}
        _retval = _UniFFILib.{{ meth.ffi_func().name() }}(
            self._handle,
            {%- for arg in meth.arguments() %}
            {{ arg.name()|lower_py(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        return {% match meth.return_type() %}{% when Some with (return_type) %}{{ "_retval"|lift_py(return_type) }}{% else %}None{% endmatch %}
    {%- endfor %}