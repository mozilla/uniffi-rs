{%- let namespace = ci.namespace_for_module_path(module_path)? %}
{%- let package_name=self.external_type_package_name(module_path, namespace) %}
{%- let fully_qualified_type_name = "{}.{}"|format(package_name, name|class_name(ci)) %}
{%- let fully_qualified_ffi_converter_name = "{}.FfiConverterType{}"|format(package_name, name) %}
{%- let fully_qualified_rustbuffer_name = "{}.RustBuffer"|format(package_name) %}
{%- let local_rustbuffer_name = "RustBuffer{}"|format(name) %}

{{- self.add_import(fully_qualified_type_name) }}
{{- self.add_import(fully_qualified_ffi_converter_name) }}
{{ self.add_import_as(fully_qualified_rustbuffer_name, local_rustbuffer_name) }}

{%- if ci.is_name_used_as_error(name) %}
{%- let class_name = name|class_name(ci) %}

object {{ class_name }}ExternalErrorHandler : UniffiRustCallStatusErrorHandler<{{ class_name }}> {
    override fun lift(error_buf: RustBuffer): {{ class_name }} =
        {{ fully_qualified_type_name }}.ErrorHandler.lift(
            {{ local_rustbuffer_name }}(error_buf.capacity, error_buf.len, error_buf.data)
        )
}

{%- endif %}
