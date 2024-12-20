{#
// Forward work to `uniffi_macros` This keeps macro-based and UDL-based generated code consistent.
#}

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
