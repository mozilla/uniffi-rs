{#
// Forward work to `uniffi_macros` This keeps macro-based and UDL-based generated code consistent.
#}

{%- let uniffi_trait_methods = rec.uniffi_trait_methods() %}
{%- if uniffi_trait_methods.debug_fmt.is_some() %}
#[::uniffi::export_for_udl_derive(Debug)]
{%- endif %}
{%- if uniffi_trait_methods.display_fmt.is_some() %}
#[::uniffi::export_for_udl_derive(Display)]
{%- endif %}
{%- if uniffi_trait_methods.hash_hash.is_some() %}
#[::uniffi::export_for_udl_derive(Hash)]
{%- endif %}
{%- if uniffi_trait_methods.ord_cmp.is_some() %}
#[::uniffi::export_for_udl_derive(Ord)]
{%- endif %}
{%- if uniffi_trait_methods.eq_eq.is_some() %}
#[::uniffi::export_for_udl_derive(Eq)]
{%- endif %}
{%- if rec.remote() %}
#[::uniffi::udl_remote(Record)]
{%- else %}
#[::uniffi::udl_derive(Record)]
{%- endif %}
struct r#{{ rec.name() }} {
    {%- for field in rec.fields() %}
    r#{{ field.name() }}: {{ field.as_type().borrow()|type_rs }},
    {%- endfor %}
}
