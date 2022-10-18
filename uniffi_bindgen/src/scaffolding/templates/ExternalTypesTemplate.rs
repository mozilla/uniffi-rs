// Support for external types.

// Types with an external `FfiConverter`...
{% for (name, crate_name) in ci.iter_external_types() %}
// `{{ name }}` is defined in `{{ crate_name }}`
use {{ crate_name|crate_name_rs }}::FfiConverterType{{ name }};
{% endfor %}

// For custom scaffolding types we need to generate an FfiConverterType based on the
// From / TryFrom implementations that the library supplies
{% for (name, builtin) in ci.iter_custom_types() %}
{% let builtin_type = builtin|type_rs -%}
// Type `{{ name }}` wraps a `{{ builtin.canonical_name() }}`
#[doc(hidden)]
pub struct FfiConverterType{{ name }};

unsafe impl ::uniffi::FfiConverter for FfiConverterType{{ name }} {
    type RustType = r#{{ name }};
    type FfiType = {{ FFIType::from(builtin).borrow()|type_ffi }};

    fn lower(obj: {{ name }}) -> Self::FfiType {
        <{{ builtin_type }} as ::uniffi::FfiConverter>::lower(
            <{{ builtin_type }} as ::std::convert::From<r#{{ name }}>>::from(obj),
        )
    }

    fn try_lift(v: Self::FfiType) -> ::uniffi::Result<r#{{ name }}> {
        <r#{{ name }} as ::std::convert::TryFrom<{{ builtin_type }}>>::try_from(
            <{{ builtin_type }} as ::uniffi::FfiConverter>::try_lift(v)?,
        ).map_err(::std::convert::From::from)
    }

    fn write(obj: {{ name }}, buf: &mut Vec<u8>) {
        <{{ builtin_type }} as ::uniffi::FfiConverter>::write(
            <{{ builtin_type }} as ::std::convert::From<r#{{ name }}>>::from(obj),
            buf,
        );
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::Result<r#{{ name }}> {
        <r#{{ name }} as ::std::convert::TryFrom<{{ builtin_type }}>>::try_from(
            <{{ builtin_type }} as ::uniffi::FfiConverter>::try_read(buf)?,
        ).map_err(::std::convert::From::from)
    }
}
{%- endfor -%}
