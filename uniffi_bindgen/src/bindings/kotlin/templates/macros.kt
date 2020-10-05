{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_kt` in `lower_kt`)
#}

{%- macro to_ffi_call(func) -%}
rustCall(
    {%- match func.throws() %}
    {%- when Some with (e) %}
    {{-e}}.ByReference()
    {%- else %}
    InternalError.ByReference()
    {%- endmatch %}
) { err ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}err)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) %}
rustCall(
    {%- match func.throws() %}
    {%- when Some with (e) %}
    {{e}}.ByReference()
    {%- else %}
    InternalError.ByReference()
    {%- endmatch %}
) { err ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) %}{% if func.arguments().len() > 0 %}, {% endif %}err)
}
{%- endmacro %}


{%- macro _arg_list_ffi_call(func) %}
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
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_kt }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{% macro arg_list_protocol(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_kt }}: {{ arg.type_()|type_kt -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}
{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name() }}: {{ arg.type_()|type_ffi -}}
        {%- if loop.last %}{% else %},{% endif %}
    {%- endfor %}
    {% if func.arguments().len() > 0 %},{% endif %} uniffi_out_err: Structure.ByReference
{%- endmacro -%}
