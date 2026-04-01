// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::identity_op)]
#[allow(clippy::needless_question_mark)]
#[allow(clippy::unit_arg)]
#[allow(clippy::too_many_arguments)]
#[allow(unused)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use ::uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;
    use ::uniffi_bindgen_kotlin_jni_runtime::uniffi;
    use ::uniffi_bindgen_kotlin_jni_runtime::JniResultExt;

    {%- filter indent(4) %}{% include "lift_lower.rs" %}{% endfilter %}

    {%- for (jni_method_name, callable) in root.jni_methods() %}
    {%- filter indent(4) %}{% include "jni_method.rs" %}{% endfilter %}
    {%- endfor %}

    {%- for type_def in root.ffi_type_definitions() %}
    {%- match type_def %}
    {%- when TypeDefinition::Record(rec) %}
    {%- filter indent(4) %}{% include "record.rs" %}{% endfilter %}
    {%- endmatch %}
    {%- endfor %}
}
