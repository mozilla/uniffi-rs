{%- for arg in callable.arguments %}
{%- if let Some(default) = arg.default %}
if {{ arg.name }} is _DEFAULT:
    {{ arg.name }} = {{ default.py_default }}
{%- endif %}
{{ arg.ty.ffi_converter_name }}.check_lower({{ arg.name }})
{% endfor -%}

 _uniffi_lowered_args = (
    {%- if callable.is_method() %}
    self._uniffi_clone_handle(),
    {%- endif %}
    {%- for arg in callable.arguments %}
    {{ arg.ty.ffi_converter_name }}.lower({{ arg.name }}),
    {%- endfor %}
)

{%- match callable.return_type.ty %}
{%- when Some(return_type) %}
_uniffi_lift_return = {{ return_type.ffi_converter_name }}.lift
{%- when None %}
_uniffi_lift_return = lambda val: None
{%- endmatch %}

{%- match callable.throws_type.ty %}
{%- when Some(e) %}
{%-    match e.ty %}
{%-    when Type::Enum { .. } %}
_uniffi_error_converter = {{ e.ffi_converter_name }}
{%-    when Type::Interface { .. } %}
_uniffi_error_converter = {{ e.ffi_converter_name }}__as_error
{%-    else %}
_uniffi_error_converter = "UNSUPPORTED ERROR TYPE: {{"{:?}"|format(e) }}"
{%-    endmatch %}
{%- when None %}
_uniffi_error_converter = None
{%- endmatch %}


{%- match callable.async_data %}
{%- when None %}
_uniffi_ffi_result = _uniffi_rust_call_with_error(
    _uniffi_error_converter,
    _UniffiLib.{{ callable.ffi_func.0 }},
    *_uniffi_lowered_args,
)
{%- match callable.kind %}
{%- when CallableKind::Constructor { primary: true, .. } %}
self._handle = _uniffi_ffi_result
{%- when CallableKind::Constructor { primary: false, .. } %}
return cls._uniffi_make_instance(_uniffi_ffi_result)
{%- else %}
return _uniffi_lift_return(_uniffi_ffi_result)
{%- endmatch %}
{%- when Some(async_data) %}
return await _uniffi_rust_call_async(
    _UniffiLib.{{ callable.ffi_func.0 }}(*_uniffi_lowered_args),
    _UniffiLib.{{ async_data.ffi_rust_future_poll.0 }},
    _UniffiLib.{{ async_data.ffi_rust_future_complete.0 }},
    _UniffiLib.{{ async_data.ffi_rust_future_free.0 }},
    _uniffi_lift_return,
    _uniffi_error_converter,
)
{%- endmatch %}
