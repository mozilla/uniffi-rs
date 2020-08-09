{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_py` in `lower_py`)
#}

{%- macro to_ffi_call(func) -%}
{%- match func.throws() -%}
{%- when Some with (e) -%}
_RustErrorHelper.try_raise(_RustErrorHelperPartial(_UniFFILib.{{ func.ffi_func().name() }},{% call _arg_list_ffi_call(func) -%}))
{%- else -%}
_UniFFILib.{{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%})
{%- endmatch -%}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
{%- match func.throws() -%}
{%- when Some with (e) -%}
_RustErrorHelper.try_raise(_RustErrorHelperPartial(
    _UniFFILib.{{ func.ffi_func().name() }},{{- prefix }}{% if func.arguments().len() > 0 %},{% endif %}{% call _arg_list_ffi_call(func) %})
))
{%- else -%}
_UniFFILib.{{ func.ffi_func().name() }}(
    {{- prefix }}{% if func.arguments().len() > 0 %},{% endif %}{% call _arg_list_ffi_call(func) %})
)
{%- endmatch -%}
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
    {%- if func.has_out_err() -%}ctypes.POINTER(RustError),{%- endif -%}
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
