// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use ::uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;
    use ::uniffi_bindgen_kotlin_jni_runtime::uniffi;

    {%- for (jni_method_name, callable) in root.jni_methods() %}
    {%- filter indent(4) %}{% include "jni_method.rs" %}{% endfilter %}
    {%- endfor %}
}
