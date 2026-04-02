// Skip clippy checks for the generate bindings
#[allow(clippy::unused_unit)]
#[allow(clippy::let_unit_value)]
#[allow(unused)]
mod uniffi_bindgen_kotlin_jni_scaffolding {
    use ::uniffi_bindgen_kotlin_jni_runtime as uniffi_jni;
    use ::uniffi_bindgen_kotlin_jni_runtime::uniffi;

    {% filter indent(4) %}{% include "shared.rs" %}{% endfilter %}

    {%- for package in root.packages %}
    {%- for type_def in package.type_definitions %}
    {%- match type_def %}
    {%- when TypeDefinition::Record(rec) %}
    {%- filter indent(4) %}{% include "record.rs" %}{% endfilter %}
    {%- when TypeDefinition::Enum(en) %}
    {%- filter indent(4) %}{% include "enum.rs" %}{% endfilter %}
    {%- endmatch %}
    {%- endfor %}

    {%- for func in package.functions %}
    {%- filter indent(4) %}{% include "function.rs" %}{% endfilter %}
    {%- endfor %}
    {%- endfor %}
}
