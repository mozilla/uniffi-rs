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

// We generate support for each Custom Type and the builtin type it uses.
{%- for (name, builtin) in ci.iter_custom_types() %}
::uniffi::custom_type!(r#{{ name }}, {{builtin|type_rs}});
{%- endfor -%}
