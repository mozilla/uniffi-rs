{# 
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_rs_call` (we use  `var_name_swift` in `lower_swift`)
#}

{%- macro to_rs_call(func) -%}
{% match func.throws() %}
{% when Some with (e) %}
try rustCall({{e}}.NoError) { err  in
    {{ func.ffi_func().name() }}({% call _arg_list_rs_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}err)
}
{% else %}
{{ func.ffi_func().name() }}({% call _arg_list_rs_call(func) -%})
{% endmatch %}
{%- endmacro -%}

{%- macro to_rs_call_with_prefix(prefix, func) -%}
{% match func.throws() %}
{% when Some with (e) %}
try rustCall({{e}}.NoError) { err  in
    {{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_rs_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}err
    )
}
{% else %}
{{ func.ffi_func().name() }}(
        {{- prefix }}{% if func.arguments().len() > 0 %},{% endif %}{% call _arg_list_rs_call(func) -%}
)
{% endmatch %}
{%- endmacro -%}

{%- macro _arg_list_rs_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|lower_swift(arg.type_()) }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Swift declarations of methods, functions and constructors.
// Note the var_name_swift and decl_swift filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_swift }}: {{ arg.type_()|decl_swift -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_c filters.
-#}
{%- macro arg_list_rs_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.type_()|decl_c }} {{ arg.name() -}}
        {% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
    {% if func.has_out_err() %}{% if func.arguments().len() > 0 %},{% endif %}NativeRustError *_Nonnull out_err{% endif %}

{%- endmacro -%}

{%- macro throws(func) %}
{% match func.throws() %}{% when Some with (e) %}throws{% else %}{% endmatch %}
{%- endmacro -%}

{%- macro try(func) %}
{% match func.throws() %}{% when Some with (e) %}try{% else %}try!{% endmatch %}
{%- endmacro -%}
