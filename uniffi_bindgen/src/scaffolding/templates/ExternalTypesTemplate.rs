// Support for external types.

// Types with an external `FfiConverter`...
{% for (name, crate_name, kind) in ci.iter_external_types() %}
// The FfiConverter for `{{ name }}` is defined in `{{ crate_name }}`
{%- match kind %}
{%- when ExternalKind::DataClass %}
::uniffi::ffi_converter_forward!(r#{{ name }}, ::{{ crate_name|crate_name_rs }}::UniFfiTag, crate::UniFfiTag);
{%- when ExternalKind::Interface %}
::uniffi::ffi_converter_forward!(::std::sync::Arc<r#{{ name }}>, ::{{ crate_name|crate_name_rs }}::UniFfiTag, crate::UniFfiTag);
{%- endmatch %}
{%- endfor %}

// For custom scaffolding types we need to generate an FfiConverter impl based on the
// UniffiCustomTypeConverter implementation that the library supplies
{% for (name, builtin) in ci.iter_custom_types() %}

// Type `{{ name }}` wraps a `{{ builtin|debug }}`
#[::uniffi::ffi_converter_custom_type(builtin = {{ builtin|type_rs }}, tag = crate::UniFfiTag)]
struct r#{{ name }} { }

{%- endfor -%}
