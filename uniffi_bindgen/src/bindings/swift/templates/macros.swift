{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_swift` in `lower_swift`)
#}

{%- macro to_ffi_call(func) -%}
{% call try(func) %} rustCall(
    {% match func.throws() %}
    {% when Some with (e) %}
    {{e}}.NoError
    {% else %}
    InternalError.unknown()
    {% endmatch %}
) { err in
    {{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}err)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
{% call try(func) %} rustCall(
    {%- match func.throws() %}
    {%- when Some with (e) %}
    {{e}}.NoError
    {%- else %}
    InternalError.unknown()
    {%- endmatch %}
) { err in
    {{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}err
    )
}
{%- endmacro %}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|lower_swift(arg.type_()) }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Swift declarations of methods, functions and constructors.
// Note the var_name_swift and type_swift filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_swift }}: {{ arg.type_()|type_swift -}}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_swift }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}


{% macro arg_list_protocol(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_swift }}: {{ arg.type_()|type_swift -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}


{#-
// Arglist as used in the _UniFFILib function declations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.type_()|type_ffi }} {{ arg.name() -}}
        {% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
    {% if func.arguments().len() > 0 %},{% endif %}NativeRustError *_Nonnull out_err

{%- endmacro -%}

{%- macro throws(func) %}
{%- match func.throws() %}{% when Some with (e) %}throws{% else %}{% endmatch %}
{%- endmacro -%}

{%- macro try(func) %}
{%- match func.throws() %}{% when Some with (e) %}try{% else %}try!{% endmatch %}
{%- endmacro -%}
