{%- for type_node in root.lift_kt_from_rust_types() %}
static {{ type_node.lift_kt_from_rust_var() }}: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
    c"uniffi/UniffiKt",
    c"{{ type_node.lift_fn_kt() }}",
    c"{{ type_node.lift_fn_jni_signature() }}",
);
{%- endfor %}
