{% import "macros.py" as py %}
{%- let obj = self.inner() %}

class {{ obj.name()|class_name_py }}(object):
    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::coerce_args_extra_indent(cons) %}
        self._pointer = {% call py::to_ffi_call(cons) %}
    {%- when None %}
    {%- endmatch %}

    def __del__(self):
        # In case of partial initialization of instances.
        pointer = getattr(self, "_pointer", None)
        if pointer is not None:
            rust_call(_UniFFILib.{{ obj.ffi_object_free().name() }}, pointer)

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _make_instance_(cls, pointer):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required pointer.
        inst = cls.__new__(cls)
        inst._pointer = pointer
        return inst

    {% for cons in obj.alternate_constructors() -%}
    @classmethod
    def {{ cons.name()|fn_name_py }}(cls, {% call py::arg_list_decl(cons) %}):
        {%- call py::coerce_args_extra_indent(cons) %}
        # Call the (fallible) function before creating any half-baked object instances.
        pointer = {% call py::to_ffi_call(cons) %}
        return cls._make_instance_(pointer)
    {% endfor %}

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        _retval = {% call py::to_ffi_call_with_prefix("self._pointer", meth) %}
        return {{ "_retval"|lift_py(return_type) }}

    {%- when None -%}
    def {{ meth.name()|fn_name_py }}(self, {% call py::arg_list_decl(meth) %}):
        {%- call py::coerce_args_extra_indent(meth) %}
        {% call py::to_ffi_call_with_prefix("self._pointer", meth) %}
    {% endmatch %}
    {% endfor %}

    @classmethod
    def _read(cls, buf):
        ptr = buf.readU64()
        if ptr == 0:
            raise InternalError("Raw pointer value was null")
        return cls._lift(ptr)

    @classmethod
    def _write(cls, value, buf):
        if not isinstance(value, {{ obj.name()|class_name_py }}):
            raise TypeError("Expected {{ obj.name()|class_name_py }} instance, {} found".format(value.__class__.__name__))
        buf.writeU64(value._lower())

    @classmethod
    def _lift(cls, pointer):
        return cls._make_instance_(pointer)

    def _lower(self):
        return self._pointer
