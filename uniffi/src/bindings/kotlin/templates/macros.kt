{# 
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_rs_call` (we use  `var_name_kt` in `lower_kt`)
#}

{%- macro to_rs_call(func) -%}
{%- match func.throws() %}
{%- when Some with (e) -%}
    rustCall({{e}}.ByReference()) { e -> 
        _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}({% call _arg_list_rs_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}e)
    }
{%- else -%}
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}({% call _arg_list_rs_call(func) -%})
{%- endmatch %}
{%- endmacro -%}

{%- macro to_rs_call_with_prefix(prefix, func) %}
{%- match func.throws() %}
{%- when Some with (e) -%}
    rustCall({{e}}.ByReference()) { e -> 
        _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
            {{- prefix }}, {% call _arg_list_rs_call(func) %}{% if func.arguments().len() > 0 %}, {% endif %}e)
    }
{%- else -%}
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
        {{- prefix }}{% if func.arguments().len() > 0 %}, {% endif %}{% call _arg_list_rs_call(func) %})
{%- endmatch %}
{%- endmacro %}


{%- macro _arg_list_rs_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|lower_kt(arg.type_()) }}
        {%- if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in kotlin declarations of methods, functions and constructors.
// Note the var_name_kt and type_kt filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_kt }}: {{ arg.type_()|type_kt -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_c filters.
-#}
{%- macro arg_list_rs_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name() }}: {{ arg.type_()|type_c -}}
        {%- if loop.last %}{% else %},{% endif %}
    {%- endfor %}
    {% if func.has_out_err() %}{% if func.arguments().len() > 0 %},{% endif %} e: Structure.ByReference{% endif %}
{%- endmacro -%}
