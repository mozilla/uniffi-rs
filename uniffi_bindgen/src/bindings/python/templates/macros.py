{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_py` in `lower_py`)
#}

{%- macro to_ffi_call(func) -%}

rust_call_with_error(
    {%- match func.throws() -%}
    {%- when Some with (e) -%}
    {{ e|class_name_py }}
    {%- else -%}
    InternalError
    {%- endmatch -%},
    _UniFFILib.{{ func.ffi_func().name() }}{% if func.arguments().len() > 0 %},{% endif %}{% call _arg_list_ffi_call(func) -%}
)
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
rust_call_with_error(
    {%- match func.throws() -%}
    {%- when Some with (e) -%}
    {{ e|class_name_py }}
    {%- else -%}
    InternalError
    {%- endmatch -%},
    _UniFFILib.{{ func.ffi_func().name() }},{{- prefix }}{% if func.arguments().len() > 0 %},{% endif %}{% call _arg_list_ffi_call(func) %})
)
{%- endmacro -%}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|lower_py(arg.type_()) }}
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
        {%- if !loop.last %},{% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.type_()|type_ffi }},{##}
    {%- endfor %}
    ctypes.POINTER(RustError)
{%- endmacro -%}

{%- macro coerce_args(func) %}
    {%- for arg in func.arguments() %}
    {{ arg.name()|coerce_py(arg.type_()) -}}
    {% endfor -%}
{%- endmacro -%}

{%- macro coerce_args_extra_indent(func) %}
        {%- for arg in func.arguments() %}
        {{ arg.name()|coerce_py(arg.type_()) }}
        {%- endfor %}
{%- endmacro -%}
