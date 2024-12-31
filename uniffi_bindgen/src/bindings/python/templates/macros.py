{# Generates a definition for a function/method/constructor #}
{%- macro define_callable(callable) -%}
{{ callable|def }} {{ callable.name }}({{ callable|arg_list }}){% if !callable.is_primary_constructor() %} -> {{ callable|return_type }}{% endif %}:
    {{ callable.docstring|docindent(4) -}}
    {%- for arg in callable.arguments() %}
    {%- if let Some(literal) = arg.default %}
    if {{ arg.name }} is _DEFAULT:
        {{ arg.name }} = {{ literal }}
    {%- endif %}
    {{ arg|check_lower_fn }}({{ arg.name }})
    {%- endfor %}

    {%- if let Some(async_data) = callable.async_data() %}
    _uniffi_return = await _uniffi_rust_call_async(
        _UniffiLib.{{ callable.ffi_func() }}(
            {%- if callable.is_method() %}
            self._uniffi_clone_pointer(),
            {%- endif %}
            {%- for arg in callable.arguments() %}
            {{ arg|lower_fn }}({{ arg.name }}),
            {%- endfor %}
         ),
        _UniffiLib.{{ async_data.ffi_rust_future_poll }},
        _UniffiLib.{{ async_data.ffi_rust_future_complete }},
        _UniffiLib.{{ async_data.ffi_rust_future_free }},
        {{ callable|error_ffi_converter }},
    )
    {%- else %}
    _uniffi_return = _uniffi_rust_call_with_error(
        {{ callable|error_ffi_converter }},
        _UniffiLib.{{ callable.ffi_func() }},
        {%- if callable.is_method() %}
        self._uniffi_clone_pointer(),
        {%- endif %}
        {%- for arg in callable.arguments() %}
        {{ arg|lower_fn }}({{ arg.name }}),
        {%- endfor %}
    )
    {%- endif %}

    {%- if callable.is_primary_constructor() %}
    self._pointer = _uniffi_return
    {%- else if let Some(return_type) = &callable.return_type() %}
    return {{ return_type|lift_fn }}(_uniffi_return)
    {%- endif %}

{%- endmacro %}
