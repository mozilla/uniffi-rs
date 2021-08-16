{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call`
#}

{%- macro to_ffi_call(func) -%}
    {%- match func.throws() -%}
    {%- when Some with (e) -%}
rust_call_with_error({{ e|class_name_py }},
    {%- else -%}
rust_call(
    {%- endmatch -%}
    _UniFFILib.{{ func.ffi_func().name() }},
    {%- call _arg_list_ffi_call(func) -%}
)
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
    {%- match func.throws() -%}
    {%- when Some with (e) -%}
rust_call_with_error(
    {{ e|class_name_py }},
    {%- else -%}
rust_call(
    {%- endmatch -%}
    _UniFFILib.{{ func.ffi_func().name() }},
    {{- prefix }},
    {%- call _arg_list_ffi_call(func) -%}
)
{%- endmacro -%}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{ arg.type_()|ffi_converter_name }}.lower({{ arg.name() }})
        {%- if !loop.last %},{% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Python declarations of methods, functions and constructors.
// Note the var_name_py and type_py filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_py }}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_py }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %},{% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
    {{ arg.type_()|type_ffi }},
    {%- endfor %}
    ctypes.POINTER(RustCallStatus),
{% endmacro -%}
