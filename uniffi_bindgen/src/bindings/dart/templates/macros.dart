{#
// Template to call into rust. Used in several places.
// Variable names in `arg_list_decl` should match up with arg lists
// passed to rust via `_arg_list_ffi_call`
#}

{%- macro to_ffi_call(func) -%}
    rustCall(this,
      (status) => _{{func.name()|fn_name}}(
          {% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %}, {% endif %}
          status
      )
    )
{%- endmacro -%}

{%- macro gen_ffi_signatures_global(meth) -%}
    late final _{{meth.name()|fn_name}}Ptr = _lookup<NativeFunction<{% match meth.return_type() -%}
      {%- when Some with (return_type) -%} {{ return_type|ffi_type }} 
      {%- when None %} Void 
    {%- endmatch %} Function(
      {% call _arg_types_ffi_call(meth) %},
      Pointer<RustCallStatus>
    )>>("{{ func.ffi_func().name() }}");

    late final _{{meth.name()|fn_name}} = _{{meth.name()|fn_name}}Ptr.asFunction<{% match meth.return_type() -%}
      {%- when Some with (return_type) -%} {{ return_type|dart_ffi_type }} 
      {%- when None %} void 
    {%- endmatch %} Function(
      {% call _arg_types_ffi_lifted(meth) %},
      Pointer<RustCallStatus>
      )>();
{%- endmacro %}


{%- macro gen_ffi_signatures(meth) -%}
    late final _{{meth.name()|fn_name}}Ptr = _api._lookup<NativeFunction<{% match meth.return_type() -%}
      {%- when Some with (return_type) -%} {{ return_type|ffi_type }} 
      {%- when None %} Void 
    {%- endmatch %} Function(
        Pointer,
        {% call _arg_types_ffi_call(meth) %},
        Pointer<RustCallStatus>
      )>>("{{ func.ffi_func().name() }}");

    late final _{{meth.name()|fn_name}} = _{{meth.name()|fn_name}}Ptr.asFunction<{% match meth.return_type() -%}
      {%- when Some with (return_type) -%} {{ return_type|dart_ffi_type }} 
      {%- when None %} void 
    {%- endmatch %} Function(Pointer,
      {% call _arg_types_ffi_lifted(meth) %},
      Pointer<RustCallStatus>
      )>();
{%- endmacro %}

{%- macro to_ffi_call_with_prefix(prefix, func) -%}
    {#
      {%- match func.throws_type() %}
      {%- when Some with (e) %}
      rustCallWithError({{ e|ffi_converter_name }}.self) {
      {%- else %}
      rustCall() {
    {% endmatch %}
    #}
      _{{ func.name()|fn_name }}(
        {{- prefix }}, {% call _arg_list_ffi_call(func) -%}{% if func.arguments().len() > 0 %}, {% endif %})
{%- endmacro %}

{%- macro _arg_list_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{ arg.name()|var_name }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro -%}

{%- macro _arg_types_ffi_call(func) %}
    {%- for arg in func.arguments() %}
        {{ arg|lower_type }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro -%}


{%- macro _arg_types_ffi_lifted(func) %}
    {%- for arg in func.arguments() %}
        {{ arg|lift_type }}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro -%}



{#-
// Arglist as used in Dart declarations of methods, functions and constructors.
// Note the var_name and type_name filters.
-#}

{% macro arg_list_decl(func) %}
    {%- for arg in func.arguments() -%}
        {{ arg|lift_type }} {{ arg.name()|var_name }}
        {%- match arg.default_value() %}
        {%- when Some with(literal) %} = {{ literal|literal_dart(arg) }}
        {%- else %}
        {%- endmatch %}
        {%- if !loop.last %}, {% endif -%}
    {%- endfor %}
{%- endmacro %}

{#-
// Field lists as used in Dart declarations of Records and Enums.
// Note the var_name and type_name filters.
-#}
{% macro field_list_decl(item) %}
    {%- for field in item.fields() -%}
        {{ field.name()|var_name }}: {{ field|type_name -}}
        {%- match field.default_value() %}
            {%- when Some with(literal) %} = {{ literal|literal_dart(field) }}
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
    {%- for arg in func.arguments() %}
        {{- arg.type_().borrow()|ffi_type_name }} {{ arg.name() -}},
    {%- endfor %}
    RustCallStatus *_Nonnull out_status
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
import 'dart:ffi';
import 'Helpers.dart';
