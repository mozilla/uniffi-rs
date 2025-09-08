{%- let protocol = int.protocol %}
{%- let ffi_converter_name = int.self_type.ffi_converter_name %}
{%- include "Protocol.py" %}

class {{ int.name }}({{ int.base_classes|join(", ") }}):
    {{ int.docstring|docstring(4) }}
    _handle: ctypes.c_uint64

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
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(cls, {% include "CallableArgs.py" %}) -> {{ callable.return_type.type_name }}:
        {{ cons.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%-     endif %}
{%- endfor %}

{%- if !int.has_primary_constructor %}
    {# Define __init__ to prevent construction without a handle, which can confuse #}
    def __init__(self, *args, **kwargs):
        raise ValueError("This class has no default constructor")
{%- endif %}

    def __del__(self):
        # In case of partial initialization of instances.
        handle = getattr(self, "_handle", None)
        if handle is not None:
            _uniffi_rust_call(_UniffiLib.{{ int.ffi_func_free.0 }}, handle)

    def _uniffi_clone_handle(self):
        return _uniffi_rust_call(_UniffiLib.{{ int.ffi_func_clone.0 }}, self._handle)

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _uniffi_make_instance(cls, handle):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required handle.
        inst = cls.__new__(cls)
        inst._handle = handle
        return inst

{%- for meth in int.methods -%}
{%-     let callable = meth.callable %}
    {% if callable.is_async %}async {% endif %}def {{ callable.name }}(self, {% include "CallableArgs.py" %}) -> {{ callable.return_type.type_name }}:
        {{ meth.docstring|docstring(8) -}}
        {%- filter indent(8) %}
        {%- include "CallableBody.py" %}
        {%- endfilter %}
{%- endfor %}

{%- let uniffi_trait_methods = int.uniffi_trait_methods %}
{% filter indent(4) %}
{% include "UniffiTraitImpls.py" -%}
{% endfilter %}

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
        # Errors are always a rust buffer holding a handle - which is a "read"
        with value.consume_with_stream() as stream:
            return {{ ffi_converter_name }}.read(stream)

    @staticmethod
    def lower(value):
        raise NotImplementedError()

{%- endif %}

{%- match int.vtable %}
{%- when None %}
{# simple case: the interface can only be implemented in Rust #}
class {{ ffi_converter_name }}:
    @staticmethod
    def lift(value: int) -> {{ int.name }}:
        return {{ int.name }}._uniffi_make_instance(value)

    @staticmethod
    def check_lower(value: {{ int.name }}):
        if not isinstance(value, {{ int.name }}):
            raise TypeError("Expected {{ int.name }} instance, {} found".format(type(value).__name__))

    @staticmethod
    def lower(value: {{ int.name }}) -> ctypes.c_uint64:
        return value._uniffi_clone_handle()

    @classmethod
    def read(cls, buf: _UniffiRustBuffer) -> {{ int.name }}:
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw handle value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ int.name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))

{%- when Some(vtable) %}
{#
 # The interface can be implemented in Rust or Python

 # * Generate a callback interface implementation to handle the Python side
 # * In the FfiConverter, check which side a handle came from to know how to handle correctly.
 #}

{%- let trait_impl=format!("_UniffiTraitImpl{}", int.name) %}
{%- include "CallbackInterfaceImpl.py" %}

class {{ ffi_converter_name }}:
    _handle_map = _UniffiHandleMap()

    @staticmethod
    def lift(value: int):
        if (value & 1) == 0:
            # Rust-generated handle, construct a new class that uses the handle to implement the
            # interface
            return {{ int.name }}._uniffi_make_instance(value)
        else:
            # Python-generated handle, get the object from the handle map
            return {{ ffi_converter_name }}._handle_map.remove(value)

    @staticmethod
    def check_lower(value: {{ protocol.name }}):
        if not isinstance(value, {{ protocol.name }}):
            raise TypeError("Expected {{ protocol.name }} subclass, {} found".format(type(value).__name__))

    @staticmethod
    def lower(value: {{ protocol.name }}):
         if isinstance(value, {{ int.name }}):
            # Rust-implementated object.  Clone the handle and return it
            return value._uniffi_clone_handle()
         else:
            # Python-implementated object, generate a new vtable handle and return that.
            return {{ ffi_converter_name }}._handle_map.insert(value)

    @classmethod
    def read(cls, buf: _UniffiRustBuffer):
        ptr = buf.read_u64()
        if ptr == 0:
            raise InternalError("Raw handle value was null")
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ protocol.name }}, buf: _UniffiRustBuffer):
        buf.write_u64(cls.lower(value))
{%- endmatch %}
