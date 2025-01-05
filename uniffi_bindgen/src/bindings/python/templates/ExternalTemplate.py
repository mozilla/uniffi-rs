{%- let namespace = ci.namespace_for_type(type_)? %}
{%- let module = python_config.module_for_namespace(namespace) -%}

# External type {{ name }}: `from {{ module }} import {{ name }}`
{%- let ffi_converter_name = "_UniffiConverterType{}"|format(name) %}
{{ self.add_import_of(module, ffi_converter_name) }}
{{ self.add_import_of(module, name) }} {#- import the type alias itself -#}

# External type {{ name }} is an error.
{%- if ci.is_name_used_as_error(name) %}
{%-     match type_ %}
{%-         when Type::Object { .. } %}
{%-             let err_ffi_converter_name = "{}__as_error"|format(ffi_converter_name) %}
{{ self.add_import_of(module, err_ffi_converter_name) }}
{%-         else %}
{%-     endmatch %}
{%- endif %}

{%- let rustbuffer_local_name = "_UniffiRustBuffer{}"|format(name) %}
{{ self.add_import_of_as(module, "_UniffiRustBuffer", rustbuffer_local_name) }}
