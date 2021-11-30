// Support for external types.

// Types with an external `FfiConverter`...
{% for (name, crate_name) in ci.iter_external_types() %}
// `{{ name }}` is defined in `{{ crate_name }}`
use {{ crate_name|crate_name_rs }}::FfiConverterType{{ name }};
{% endfor %}

// More complicated locally `Wrapped` types - we generate FfiConverter.
{% for (name, prim) in ci.iter_wrapped_types() %}
{% if loop.first %}

// A trait that's in our crate for our external wrapped types to implement.
pub trait UniffiCustomTypeWrapper {
    type Wrapped;
    type Error: std::error::Error;

    fn wrap(val: Self::Wrapped) -> std::result::Result<Self, Self::Error> where Self: Sized;
    fn unwrap(obj: Self) -> Self::Wrapped;
}

{%- endif -%}

// Type `{{ name }}` wraps a `{{ prim.canonical_name() }}`
#[doc(hidden)]
pub struct FfiConverterType{{ name }};

unsafe impl uniffi::FfiConverter for FfiConverterType{{ name }} {
    type RustType = {{ name }};
    type FfiType = {{ prim.into()|type_ffi }};
    type Error = <{{name}} as UniffiCustomTypeWrapper>::Error;

    fn lower(obj: {{ name }} ) -> Self::FfiType {
        <{{ prim|type_rs }} as uniffi::FfiConverter>::lower(<{{ name }} as UniffiCustomTypeWrapper>::unwrap(obj))
    }

    fn try_lift(v: Self::FfiType) -> std::result::Result<{{ name }}, Self::Error> {
        <{{ name }} as UniffiCustomTypeWrapper>::wrap(<{{ prim|type_rs }} as uniffi::FfiConverter>::try_lift(v).unwrap())
    }

    fn write(obj: {{ name }}, buf: &mut Vec<u8>) {
        <{{ prim|type_rs }} as uniffi::FfiConverter>::write(<{{ name }} as UniffiCustomTypeWrapper>::unwrap(obj), buf);
    }

    fn try_read(buf: &mut &[u8]) -> std::result::Result<{{ name }}, Self::Error> {
        <{{ name }} as UniffiCustomTypeWrapper>::wrap(<{{ prim|type_rs }} as uniffi::FfiConverter>::try_read(buf).unwrap()) // FIXME: Shouldn't unwrap
    }
}
{%- endfor -%}
