{%- if let Some(vtable) = interface.vtable %}
{% include "VTable.py" %}
{%- endif %}

class {{ interface.name }}({{ interface.base_classes|join(", ") }}):
    {{ interface.docstring|docindent(4) -}}
    _pointer: ctypes.c_void_p

    {%- match interface.primary_constructor() %}
    {%- when Some(cons) %}
    {% filter indent(4) %}{% call py::define_callable(cons) %}{% endfilter %}
    {%- when None %}
    {# Define __init__ to prevent construction without a pointer, which can confuse #}
    def __init__(self):
        {%- if interface.had_async_constructor %}
        raise ValueError("Async constructors are not supported in Python, call the `new` classmethod instead.")
        {%- else %}
        raise ValueError("This class has no default constructor")
        {%- endif %}
    {%- endmatch %}

    def __del__(self):
        # In case of partial initialization of instances.
        pointer = getattr(self, "_pointer", None)
        if pointer is not None:
            _uniffi_rust_call(_UniffiLib.{{ interface.ffi_free }}, pointer)

    def _uniffi_clone_pointer(self):
        return _uniffi_rust_call(_UniffiLib.{{ interface.ffi_clone }}, self._pointer)

    # Used by the lift function to construct a new instance
    @classmethod
    def _make_instance_(cls, pointer):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required pointer.
        inst = cls.__new__(cls)
        inst._pointer = pointer
        return inst

{%- for cons in interface.alternate_constructors() %}
    @staticmethod
    {% filter indent(4) %}{% call py::define_callable(cons) %}{% endfilter %}
{% endfor %}

{%- for meth in interface.methods %}
    {% filter indent(4) %}{% call py::define_callable(meth) %}{% endfilter %}
{%- endfor %}

{%- for uniffi_trait in interface.uniffi_traits %}
{%- match uniffi_trait %}
{%- when UniffiTrait::Eq { eq, ne } %}
{# Special-case these to return NotImplemented when the wrong type is passed in #}
    def __eq__(self, other: object) -> bool:
        if not isinstance(other, {{ interface.name }}):
            return NotImplemented

        return {{ eq.return_type().as_ref().unwrap()|lift_fn }}(
            _uniffi_rust_call_with_error(
                None,
                _UniffiLib.{{ eq.ffi_func() }},
                self._uniffi_clone_pointer(),
                {{ interface|lower_fn }}(other),
            )
        )

    def __ne__(self, other: object) -> bool:
        if not isinstance(other, {{ interface.name }}):
            return NotImplemented

        return {{ ne.return_type().as_ref().unwrap()|lift_fn }}(
            _uniffi_rust_call_with_error(
                None,
                _UniffiLib.{{ ne.ffi_func() }},
                self._uniffi_clone_pointer(),
                {{ interface|lower_fn }}(other),
            )
        )
{%- else %} 
{%- for trait_method in uniffi_trait.methods() %}
    {% filter indent(4) %}{% call py::define_callable(trait_method) %}{% endfilter %}
{%- endfor %}
{%- endmatch %}
{%- endfor %}

{# Interface as error #}
{%- if interface.is_used_as_error() %}
{# Due to some mismatches in the ffi converter mechanisms, errors are forced to be a RustBuffer #}
class {{ ffi_converter_name }}__as_error(_UniffiConverterRustBuffer):
    @classmethod
    def read(cls, buf):
        raise NotImplementedError()

    @classmethod
    def write(cls, value, buf):
        raise NotImplementedError()

    @staticmethod
    def lift(value):
        # Errors are always a rust buffer holding a pointer - which is a "read"
        with value.consume_with_stream() as stream:
            return {{ ffi_converter_name }}.read(stream)

    @staticmethod
    def lower(value):
        raise NotImplementedError()

{%- endif %}

class {{ ffi_converter_name }}:
    {%- if interface.has_callback_interface() %}
    _handle_map = _UniffiHandleMap()
    {%- endif %}

    @staticmethod
    def lift(value: int):
        return {{ interface.name }}._make_instance_(value)

    @staticmethod
    def check_lower(value: {{ interface.protocol_name }}):
        {%- if interface.has_callback_interface() %}
        pass
        {%- else %}
        if not isinstance(value, {{ interface.name }}):
            raise TypeError("Expected {{ interface.name }} instance, {} found".format(type(value).__name__))
        {%- endif %}

    @staticmethod
    def lower(value: {{ interface.protocol_name }}):
        {%- if interface.has_callback_interface() %}
        return {{ ffi_converter_name }}._handle_map.insert(value)
        {%- else %}
        if not isinstance(value, {{ interface.name }}):
            raise TypeError("Expected {{ interface.name }} instance, {} found".format(type(value).__name__))
        return value._uniffi_clone_pointer()
        {%- endif %}

    @classmethod
    def read(cls, buf: _UniffiRustBuffer):
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw pointer value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ interface.protocol_name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))
