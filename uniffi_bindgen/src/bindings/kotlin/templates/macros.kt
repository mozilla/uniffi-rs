{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call`
#}

{%- macro to_ffi_call(func) -%}
    {%- match func.throws() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|exception_name_kt}})
    {%- else %}
    rustCall()
    {%- endmatch %} { status ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}({% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %},{% endif %}status)
}
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) %}
    {%- match func.throws() %}
    {%- when Some with (e) %}
    rustCallWithError({{ e|exception_name_kt}})
    {%- else %}
    rustCall()
    {%- endmatch %} { status ->
    _UniFFILib.INSTANCE.{{ func.ffi_func().name() }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) %}{% if func.arguments().len() > 0 %}, {% endif %}status)
}
{%- endmacro %}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{ arg.type_()|ffi_converter_name }}.lower({{ arg.name()|var_name_kt }})
        {%- if !loop.last %}, {% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in kotlin declarations of methods, functions and constructors.
// Note the var_name_kt and type_kt filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {% let arg_type = arg.type_() -%}
        {{ arg.name()|var_name_kt }}: {{ arg_type|type_kt -}}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_kt(arg_type) }}
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
        {{- arg.name() }}: {{ arg.type_()|type_ffi -}},
    {%- endfor %}
    uniffi_out_err: RustCallStatus
{%- endmacro -%}

{#
// Add annotation if there are unsigned types
// Works for MemberDeclarations that have declared a contains_unsigned_types() method.
#}
{%- macro unsigned_types_annotation(member) -%}
{% if member.contains_unsigned_types() %}@ExperimentalUnsignedTypes{% endif %}
{%- endmacro -%}

// Macro for destroying fields
{%- macro destroy_fields(member) %}
    Disposable.destroy(
    {%- for field in member.fields() %}
        this.{{ field.name()|var_name_kt }}{%- if !loop.last %}, {% endif -%}
    {% endfor -%})
{%- endmacro -%}

{%- macro ffi_function_definition(func) %}
fun {{ func.name() }}(
    {%- call arg_list_ffi_decl(func) %}
){%- match func.return_type() -%}{%- when Some with (type_) %}: {{ type_|type_ffi }}{% when None %}: Unit{% endmatch %}
{% endmacro %}
