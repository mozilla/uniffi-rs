// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;

    {%- for package in root.packages %}
    {%- for func in package.functions %}
    {%- filter indent(4) %}{% include "function.rs" %}{% endfilter %}
    {%- endfor %}
    {%- endfor %}
}
