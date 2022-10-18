// Support for external types.

// For custom scaffolding types we need to generate an FfiConverterType based on the
// From / TryFrom implementations that the library supplies
{% for (name, builtin) in ci.iter_custom_types() %}
{% let builtin_type = builtin|type_rs -%}

unsafe impl ::uniffi::FfiConverter for r#{{ name }} {
    type FfiType = {{ FFIType::from(builtin).borrow()|type_ffi }};

    fn lower(obj: {{ name }}) -> Self::FfiType {
        <{{ builtin_type }} as ::uniffi::FfiConverter>::lower(
            <{{ builtin_type }} as ::std::convert::From<Self>>::from(obj),
        )
    }

    fn try_lift(v: Self::FfiType) -> ::uniffi::Result<Self> {
        <Self as ::std::convert::TryFrom<{{ builtin_type }}>>::try_from(
            <{{ builtin_type }} as ::uniffi::FfiConverter>::try_lift(v)?,
        ).map_err(::std::convert::From::from)
    }

    fn write(obj: {{ name }}, buf: &mut Vec<u8>) {
        <{{ builtin_type }} as ::uniffi::FfiConverter>::write(
            <{{ builtin_type }} as ::std::convert::From<Self>>::from(obj),
            buf,
        );
    }

    fn try_read(buf: &mut &[u8]) -> uniffi::Result<Self> {
        <Self as ::std::convert::TryFrom<{{ builtin_type }}>>::try_from(
            <{{ builtin_type }} as ::uniffi::FfiConverter>::try_read(buf)?,
        ).map_err(::std::convert::From::from)
    }
}
{%- endfor -%}
