{#
// Forward work to `uniffi_macros` This keeps macro-based and UDL-based generated code consistent.
#}

{%- let uniffi_trait_methods = e.uniffi_trait_methods() %}
{%- if uniffi_trait_methods.debug_fmt.is_some() %}
#[uniffi::export_for_udl_derive(Debug)]
{%- endif %}
{%- if uniffi_trait_methods.debug_fmt.is_some() %}
#[uniffi::export_for_udl_derive(Display)]
{%- endif %}
{%- if uniffi_trait_methods.hash_hash.is_some() %}
#[uniffi::export_for_udl_derive(Hash)]
{%- endif %}
{%- if uniffi_trait_methods.ord_cmp.is_some() %}
#[uniffi::export_for_udl_derive(Ord)]
{%- endif %}
{%- if uniffi_trait_methods.eq_eq.is_some() %}
#[uniffi::export_for_udl_derive(Eq)]
{%- endif %}
{%- if e.remote() %}
#[::uniffi::udl_remote(Enum)]
{%- else %}
#[::uniffi::udl_derive(Enum)]
{%- endif %}
{%- if e.is_non_exhaustive() %}
#[non_exhaustive]
{%- endif %}
enum r#{{ e.name() }} {
    {%- for variant in e.variants() %}
    r#{{ variant.name() }} {
        {%- for field in variant.fields() %}
        r#{{ field.name() }}: {{ field.as_type().borrow()|type_rs }},
        {%- endfor %}
    },
    {%- endfor %}
}
