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

{% macro mock_var_prefix(func) -%}
    {{ func.name()|fn_name }}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name|capitalize }}
    {%- endfor %}
{%- endmacro -%}

{#-
// Field lists as used in Swift declarations of Records and Enums.
// Note the var_name and type_name filters.
-#}
{% macro field_list_decl(item) %}
    {%- for field in item.fields() -%}
        {%- call docstring(field, 8) %}
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


{%- macro async(func) %}
{%- if func.is_async() %}async {% endif %}
{%- endmacro -%}

{%- macro throws(func) %}
{%- if func.throws() %}throws {% endif %}
{%- endmacro -%}

{%- macro try(func) %}
{%- if func.throws() %}try {% else %}try! {% endif %}
{%- endmacro -%}

{%- macro docstring_value(maybe_docstring, indent_spaces) %}
{%- match maybe_docstring %}
{%- when Some(docstring) %}
{{ docstring|docstring(indent_spaces) }}
{%- else %}
{%- endmatch %}
{%- endmacro %}

{%- macro docstring(defn, indent_spaces) %}
{%- call docstring_value(defn.docstring(), indent_spaces) %}
{%- endmacro %}
