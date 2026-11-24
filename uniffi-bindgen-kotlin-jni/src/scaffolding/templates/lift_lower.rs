{%- for type_node in package.lift_kt_from_rust_types() %}
static {{ package.lift_kt_from_rust_var(type_node) }}: uniffi_jni::CachedStaticMethod = uniffi_jni::CachedStaticMethod::new(
    c"{{ package.jni_path() }}/uniffi/UniffiKt",
    c"{{ type_node.lift_fn_kt() }}",
    c"{{ type_node.lift_fn_jni_signature() }}",
);
{%- endfor %}
