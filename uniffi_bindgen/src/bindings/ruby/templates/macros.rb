{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call` (we use  `var_name_rb` in `lower_rb`)
#}

{#
// Returns the Ruby name for an enum variant field.
// For tuple-style fields (empty name is uniffi metadata) we generate a 
// positional name like v1, v2, ... using the 1-based loop index.
#}
{%- macro field_name(field, field_num) -%}
{%- if field.name().is_empty() -%}
values[{{- field_num - 1 -}}]
{%- else -%}
{{ field.name()|var_name_rb }}
{%- endif -%}
{%- endmacro -%}
  
{%- macro to_ffi_call(func) -%}
    {%- match func.throws_type() -%}
    {%- when Some(Type::Custom { builtin, .. }) -%}
      {%- match builtin.borrow() -%}
      {%- when Type::Enum { name, .. } -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
      {%- when Type::Object { name, .. } -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
      {%- else -%}
      {{ ci.namespace()|class_name_rb }}.rust_call
      {%- endmatch -%}
    {%- when Some(Type::Enum { name, .. }) -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
    {%- when Some(Type::Object { name, .. }) -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
    {%- else -%}
      {{ ci.namespace()|class_name_rb }}.rust_call(
    {%- endmatch -%}
    :{{ func.ffi_func().name() }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
    {%- match func.throws_type() -%}
    {%- when Some(Type::Custom { builtin, .. }) -%}
      {%- match builtin.borrow() -%}
      {%- when Type::Enum { name, .. } -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
      {%- when Type::Object { name, .. } -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
      {%- else -%}
      {{ ci.namespace()|class_name_rb }}.rust_call
      {%- endmatch -%}
    {%- when Some(Type::Enum { name, .. }) -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
    {%- when Some(Type::Object { name, .. }) -%}
      {{ ci.namespace()|class_name_rb }}.rust_call_with_error({{ name|class_name_rb }},
    {%- else -%}
      {{ ci.namespace()|class_name_rb }}.rust_call(
    {%- endmatch -%}
    :{{ func.ffi_func().name() }},
    {{- prefix }},
    {%- call _arg_list_ffi_call(func) %}{% endcall -%}
)
{%- endmacro -%}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{- arg.name()|lower_rb(arg.as_type().borrow(), config) }}
        {%- if !loop.last %},{% endif %}
    {%- endfor %}
{%- endmacro -%}

{#-
// Arglist as used in Ruby declarations of methods, functions and constructors.
// Note the var_name_rb and type_rb filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg.name()|var_name_rb }}
        {%- match arg.default_value() %}
        {%- when Some(_) %} = {{ arg|arg_default_rb }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Arglist as used in the UniFFILib function declarations.
// Note unfiltered name but type_ffi filters.
-#}
{%- macro arg_list_ffi_decl(func) %}
    [{%- for arg in func.arguments() -%}{{ arg.type_().borrow()|type_ffi }}, {% endfor -%} RustCallStatus.by_ref]
{%- endmacro -%}

{%- macro setup_args(func) %}
    {%- for arg in func.arguments() %}
    {{ arg.name() }} = {{ arg.name()|coerce_rb(ci.namespace()|class_name_rb, arg.as_type().borrow(), config) }}
    {{ arg.name()|check_lower_rb(arg.as_type().borrow(), config) }}
    {% endfor -%}
{%- endmacro -%}

{%- macro setup_args_extra_indent(meth) %}
        {%- for arg in meth.arguments() %}
        {{ arg.name() }} = {{ arg.name()|coerce_rb(ci.namespace()|class_name_rb, arg.as_type().borrow(), config) }}
        {{ arg.name()|check_lower_rb(arg.as_type().borrow(), config) }}
        {%- endfor %}
{%- endmacro -%}
