{%- let protocol = int.protocol %}
{%- let ffi_converter_name = int.self_type.ffi_converter_name %}
{%- include "Protocol.py" %}

class {{ int.name }}({{ int.base_classes|join(", ") }}):
    {{ int.docstring|docstring(4) }}
    _pointer: ctypes.c_void_p

{%- for cons in int.constructors %}
{%-     let callable = cons.callable %}
{%-     if callable.is_primary_constructor() && callable.is_async %}
    def __init__(self, *args, **kw):
        raise ValueError("async constructors not supported.")
{%-     elif callable.is_primary_constructor() %}
    def __init__(self, {% include "CallableArgs.py" %}):
        {{ cons.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%-     else %}
    @classmethod
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(cls, {% include "CallableArgs.py" %}):
        {{ cons.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%-     endif %}
{%- endfor %}

{%- if !int.has_primary_constructor %}
    {# Define __init__ to prevent construction without a pointer, which can confuse #}
    def __init__(self, *args, **kwargs):
        raise ValueError("This class has no default constructor")
{%- endif %}

    def __del__(self):
        # In case of partial initialization of instances.
        pointer = getattr(self, "_pointer", None)
        if pointer is not None:
            _uniffi_rust_call(_UniffiLib.{{ int.ffi_func_free.0 }}, pointer)

    def _uniffi_clone_pointer(self):
        return _uniffi_rust_call(_UniffiLib.{{ int.ffi_func_clone.0 }}, self._pointer)

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _uniffi_make_instance(cls, pointer):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required pointer.
        inst = cls.__new__(cls)
        inst._pointer = pointer
        return inst

{%- for meth in int.methods -%}
{%-     let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(self, {% include "CallableArgs.py" %}):
        {{ meth.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%- endfor %}

{%- for tm in int.uniffi_traits -%}
{%-     match tm %}
{%-         when UniffiTrait::Debug { fmt } %}
{%-         let callable = fmt.callable %}
    def __repr__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}
{%-         when UniffiTrait::Display { fmt } %}
{%-         let callable = fmt.callable %}
    def __str__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}

{%-         when UniffiTrait::Eq { eq, ne } %}
{%-         let callable = eq.callable %}
    def __eq__(self, other: object) -> {{ callable.return_type.type_name }}:
        if not isinstance(other, {{ int.self_type.type_name }}):
            return NotImplemented

        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}

{%-         let callable = ne.callable %}
    def __ne__(self, other: object) -> {{ callable.return_type.type_name }}:
        if not isinstance(other, {{ int.self_type.type_name }}):
            return NotImplemented
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}

{%-         when UniffiTrait::Hash { hash } %}
{%-         let callable = hash.callable %}
    def __hash__(self) -> {{ callable.return_type.type_name }}:
        {% filter indent(8) -%}
        {% include "CallableBody.py" -%}
        {% endfilter -%}
{%-     endmatch %}
{%- endfor %}

{# callback interfaces #}
{%- if let Some(vtable) = int.vtable %}
{%- let trait_impl=format!("_UniffiTraitImpl{}", int.name) %}
{% include "CallbackInterfaceImpl.py" %}
{%- endif %}

{# Objects as error #}
{%- if int.self_type.is_used_as_error %}
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
    {%- if int.vtable.is_some() %}
    _handle_map = _UniffiHandleMap()
    {%- endif %}

    @staticmethod
    def lift(value: int):
        return {{ int.name }}._uniffi_make_instance(value)

    @staticmethod
    def check_lower(value: {{ int.self_type.type_name }}):
        {%- if int.vtable.is_some() %}
        pass
        {%- else %}
        if not isinstance(value, {{ int.name }}):
            raise TypeError("Expected {{ int.name }} instance, {} found".format(type(value).__name__))
        {%- endif %}

    @staticmethod
    def lower(value: {{ protocol.name }}):
        {%- if int.vtable.is_some() %}
        return {{ ffi_converter_name }}._handle_map.insert(value)
        {%- else %}
        if not isinstance(value, {{ int.name }}):
            raise TypeError("Expected {{ int.name }} instance, {} found".format(type(value).__name__))
        return value._uniffi_clone_pointer()
        {%- endif %}

    @classmethod
    def read(cls, buf: _UniffiRustBuffer):
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw pointer value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ protocol.name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))
