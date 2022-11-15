{%- if self.external_types_in_different_modules() %}
{# This import will bring in both the type itself and the FfiConverter for it #}
{{ self.add_import(self.external_type_module_name(crate_name).borrow()) }}
{%- endif %}
