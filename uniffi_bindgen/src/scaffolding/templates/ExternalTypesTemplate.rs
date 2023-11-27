// Support for external types.

// We generate support for each Custom Type and the builtin type it uses.
{%- for (name, builtin) in ci.iter_custom_types() %}
::uniffi::custom_type!(r#{{ name }}, {{builtin|type_rs}});
{%- endfor -%}
