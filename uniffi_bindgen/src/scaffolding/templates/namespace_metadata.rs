
/// Export namespace metadata.
///
/// See `uniffi_bidgen::macro_metadata` for how this is used.
{%- let const_var = "UNIFFI_META_CONST_NAMESPACE_{}"|format(ci.namespace().to_shouty_snake_case()) %}
{%- let static_var = "UNIFFI_META_NAMESPACE_{}"|format(ci.namespace().to_shouty_snake_case()) %}

const {{ const_var }}: ::uniffi::MetadataBuffer = ::uniffi::MetadataBuffer::from_code(::uniffi::metadata::codes::NAMESPACE)
    .concat_str(module_path!()) // This is the crate name, since we're in the crate root
    .concat_str("{{ ci.namespace() }}");
#[no_mangle]
pub static {{ static_var }}: [u8; {{ const_var }}.size] = {{ const_var }}.into_array();
