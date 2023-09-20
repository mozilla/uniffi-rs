{%- let mod_name = module_path|fn_name %}

{%- let ffi_converter_name = "_UniffiConverterType{}"|format(name) %}
{{ self.add_import_of(mod_name, ffi_converter_name) }}
{{ self.add_import_of(mod_name, name) }} {#- import the type alias itself -#}

{%- let rustbuffer_local_name = "_UniffiRustBuffer{}"|format(name) %}
{{ self.add_import_of_as(mod_name, "_UniffiRustBuffer", rustbuffer_local_name) }}
