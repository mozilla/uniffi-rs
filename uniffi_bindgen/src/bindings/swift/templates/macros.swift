{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_swift` in `lower_swift`)
#}

{%- macro to_ffi_call(func) -%}
{% call try(func) %}
    {% match func.throws() %}
    {% when Some with (e) %}
    rustCallWithError({{ e|class_name_swift }}.self) {
    {% else %}
    rustCall() {
    {% endmatch %}
    {{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %}, {% endif %}$0)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
{% call try(func) %}
    {%- match func.throws() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|class_name_swift }}.self) {
    {%- else %}
    rustCall() {
    {% endmatch %}
    {{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %}, {% endif %}$0
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
        {%- when Some with(literal) %} = {{ literal|literal_swift(arg.type_()) }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Field lists as used in Swift declarations of Records and Enums.
// Note the var_name_swift and type_swift filters.
-#}
{% macro field_list_decl(item) %}
    {%- for field in item.fields() -%}
        {{ field.name()|var_name_swift }}: {{ field.type_()|type_swift -}}
        {%- match field.default_value() %}
            {%- when Some with(literal) %} = {{ literal|literal_swift(field.type_()) }}
            {%- else %}
        {%- endmatch -%}
        {% if !loop.last %}, {% endif %}
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
        {{- arg.type_()|type_ffi }} {{ arg.name() -}},
    {%- endfor %}
    RustCallStatus *_Nonnull out_status
{%- endmacro -%}

{%- macro throws(func) %}
{%- match func.throws() %}{% when Some with (e) %}throws{% else %}{% endmatch %}
{%- endmacro -%}

{%- macro try(func) %}
{%- match func.throws() %}{% when Some with (e) %}try{% else %}try!{% endmatch %}
{%- endmacro -%}
