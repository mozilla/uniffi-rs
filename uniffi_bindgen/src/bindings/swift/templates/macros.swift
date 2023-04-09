{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `arg_list_lowered`
#}

{%- macro to_ffi_call(func) -%}
    {%- call try(func) -%}
    {%- match func.throws_type() -%}
    {%- when Some with (e) -%}
        rustCallWithError({{ e|ffi_converter_name }}.lift) {
    {%- else -%}
        rustCall() {
    {%- endmatch %}
    {{ func.ffi_func().name() }}({% call arg_list_lowered(func) -%} $0)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
{% call try(func) %}
    {%- match func.throws_type() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|ffi_converter_name }}.lift) {
    {%- else %}
    rustCall() {
    {% endmatch %}
    {{ func.ffi_func().name() }}(
        {{- prefix }}, {% call arg_list_lowered(func) -%} $0
    )
}
{%- endmacro %}

{%- macro arg_list_lowered(func) %}
    {%- for arg in func.arguments() %}
        {{ arg|lower_fn }}({{ arg.name()|var_name }}),
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Swift declarations of methods, functions and constructors.
// Note the var_name and type_name filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {% if config.omit_argument_labels() %}_ {% endif %}{{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_swift(arg) }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Field lists as used in Swift declarations of Records and Enums.
// Note the var_name and type_name filters.
-#}
{% macro field_list_decl(item) %}
    {%- for field in item.fields() -%}
        {{ field.name()|var_name }}: {{ field|type_name -}}
        {%- match field.default_value() %}
            {%- when Some with(literal) %} = {{ literal|literal_swift(field) }}
            {%- else %}
        {%- endmatch -%}
        {% if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro %}


{% macro arg_list_protocol(func) %}
    {%- for arg in func.arguments() -%}
        {% if config.omit_argument_labels() %}_ {% endif %}{{ arg.name()|var_name }}: {{ arg|type_name -}}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}


{#-
// Arglist as used in the _UniFFILib function declarations.
// Note unfiltered name but ffi_type_name filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    {%- if func.arguments().len() > 0 %}
        {%- for arg in func.arguments() %}
            {{- arg.type_().borrow()|ffi_type_name }} {{ arg.name() -}}{% if !loop.last || func.has_rust_call_status_arg() %}, {% endif %}
        {%- endfor %}
        {%- if func.has_rust_call_status_arg() %}RustCallStatus *_Nonnull out_status{% endif %}
    {%- else %}
        {%- if func.has_rust_call_status_arg() %}RustCallStatus *_Nonnull out_status{%- else %}void{% endif %}
    {% endif %}
{%- endmacro -%}

{%- macro async(func) %}
{%- if func.is_async() %}async{% endif %}
{%- endmacro -%}

{%- macro throws(func) %}
{%- if func.throws() %}throws{% endif %}
{%- endmacro -%}

{%- macro try(func) %}
{%- if func.throws() %}try {% else %}try! {% endif %}
{%- endmacro -%}
