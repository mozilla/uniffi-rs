class {{ obj.name()|class_name_py }}(object):
    _instances = WeakValueDictionary()
    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::coerce_args_extra_indent(cons) %}
        self._handle = {% call py::to_ffi_call(cons) %}
        {# A constructor, by definition, returns a new object. If we really
           wanted to be thorough, we could consider asserting the handle isn't
           already in the map, but that doesn't seem to offer much value.
        #}
        self.__class__._instances[self._handle] = self
    {%- when None %}
    {%- endmatch %}

    def __del__(self):
        rust_call_with_error(
            InternalError,
            _UniFFILib.{{ obj.ffi_object_free().name() }},
            self._handle
        )

    # Used by alternative constructors or any methods which return this type.
    # May return an existing instance, or create a new one if we can't find one.
    @classmethod
    def _get_or_make_instance_(cls, handle):
        {# Look in our weak map for an instance already associated with this
           pointer. If it exists we can return it, but the pointer we have
           is a clone of an `Arc<>` so has a reference we need to destroy.
        #}
        existing = cls._instances.get(handle)
        if existing is not None:
            rust_call_with_error(
                InternalError,
                _UniFFILib.{{ obj.ffi_object_free().name() }},
                handle
            )
            return existing

        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required handle.
        inst = cls.__new__(cls)
        inst._handle = handle
        cls._instances[handle] = inst
        return inst

    {% for cons in obj.alternate_constructors() -%}
    @classmethod
    def {{ cons.name()|fn_name_py }}(cls, {% call py::arg_list_decl(cons) %}):
        {%- call py::coerce_args_extra_indent(cons) %}
        # Call the (fallible) function before creating any half-baked object instances.
        handle = {% call py::to_ffi_call(cons) %}
        return cls._get_or_make_instance_(handle)
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
