{#
// Forward work to `uniffi_macros` This keeps macro-based and UDL-based generated code consistent.
#}

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
