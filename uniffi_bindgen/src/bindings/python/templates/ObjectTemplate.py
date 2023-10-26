{%- let obj = ci|get_object_definition(name) %}
{%- let (protocol_name, impl_name) = obj|object_names %}
{%- let methods = obj.methods() %}

{% include "Protocol.py" %}

class {{ impl_name }}:
    _uniffi_handle: ctypes.c_int64

{%- match obj.primary_constructor() %}
{%-     when Some with (cons) %}
    def __init__(self, {% call py::arg_list_decl(cons) -%}):
        {%- call py::setup_args_extra_indent(cons) %}
        self._uniffi_handle = {% call py::to_ffi_call(cons) %}
{%-     when None %}
{%- endmatch %}

    def __del__(self):
        # In case of partial initialization of instances.
        handle = getattr(self, "_uniffi_handle", None)
        if handle is not None:
            _rust_call(_UniffiLib.{{ obj.ffi_object_free().name() }}, handle)

    def _uniffi_clone_handle(self):
        _rust_call(_UniffiLib.{{ obj.ffi_object_inc_ref().name() }}, self._uniffi_handle)
        return self._uniffi_handle

    # Used by alternative constructors or any methods which return this type.
    @classmethod
    def _make_instance_(cls, handle):
        # Lightly yucky way to bypass the usual __init__ logic
        # and just create a new instance with the required handle.
        inst = cls.__new__(cls)
        inst._uniffi_handle = handle
        return inst

{%- for cons in obj.alternate_constructors() %}

    @classmethod
    def {{ cons.name()|fn_name }}(cls, {% call py::arg_list_decl(cons) %}):
        {%- call py::setup_args_extra_indent(cons) %}
        uniffi_handle = {% call py::to_ffi_call(cons) %}
        return cls._make_instance_(uniffi_handle)
{% endfor %}

{%- for meth in obj.methods() -%}
    {%- call py::method_decl(meth.name()|fn_name, meth) %}
{% endfor %}

{%- for tm in obj.uniffi_traits() -%}
{%-     match tm %}
{%-         when UniffiTrait::Debug { fmt } %}
            {%- call py::method_decl("__repr__", fmt) %}
{%-         when UniffiTrait::Display { fmt } %}
            {%- call py::method_decl("__str__", fmt) %}
{%-         when UniffiTrait::Eq { eq, ne } %}
    def __eq__(self, other: object) -> {{ eq.return_type().unwrap()|type_name }}:
        if not isinstance(other, {{ type_name }}):
            return NotImplemented

        return {{ eq.return_type().unwrap()|lift_fn }}({% call py::to_ffi_call_with_prefix("self._uniffi_handle", eq) %})

    def __ne__(self, other: object) -> {{ ne.return_type().unwrap()|type_name }}:
        if not isinstance(other, {{ type_name }}):
            return NotImplemented

        return {{ ne.return_type().unwrap()|lift_fn }}({% call py::to_ffi_call_with_prefix("self._uniffi_handle", ne) %})
{%-         when UniffiTrait::Hash { hash } %}
            {%- call py::method_decl("__hash__", hash) %}
{%      endmatch %}
{% endfor %}

{%- if obj.is_trait_interface() %}
{%- let callback_handler_class = format!("UniffiCallbackInterface{}", name) %}
{%- let callback_handler_obj = format!("uniffiCallbackInterface{}", name) %}
{%- let ffi_init_callback = obj.ffi_init_callback() %}
{% include "CallbackInterfaceImpl.py" %}
{%- endif %}

class {{ ffi_converter_name }}:
    {%- if obj.is_trait_interface() %}
    _slab = UniffiSlab()
    {%- endif %}

    @staticmethod
    def lift(value: UniffiHandle):
        {%- if obj.is_trait_interface() %}
        if uniffi_handle_is_from_rust(value):
            return {{ impl_name }}._make_instance_(value)
        else:
            return {{ ffi_converter_name }}._slab.remove(value)
        {%- else %}
        return {{ impl_name }}._make_instance_(value)
        {%- endif %}

    @staticmethod
    def check(value: {{ type_name }}):
        {%- if obj.is_trait_interface() %}
        pass
        {%- else %}
        if not isinstance(value, {{ impl_name }}):
            raise TypeError("Expected {{ impl_name }} instance, {} found".format(type(value).__name__))
        {%- endif %}

    @staticmethod
    def lower(value: {{ type_name }}):
        {%- if obj.is_trait_interface() %}
        _uniffi_clone_handle = getattr(value, '_uniffi_clone_handle', None)
        if _uniffi_clone_handle is not None:
            # If we're wrapping a trait implemented in Rust, return that handle directly rather than
            # wrapping it again in Python.
            return _uniffi_clone_handle()
        else:
            return {{ ffi_converter_name }}._slab.insert(value)
        {%- else %}
        return value._uniffi_clone_handle()
        {%- endif %}

    @classmethod
    def read(cls, buf: _UniffiRustBuffer):
        ptr = buf.read_i64()
        return cls.lift(ptr)

    @classmethod
    def write(cls, value: {{ type_name }}, buf: _UniffiRustBuffer):
        buf.write_i64(cls.lower(value))
