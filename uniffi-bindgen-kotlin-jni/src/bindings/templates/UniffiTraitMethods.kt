{%- if let Some(to_string) = uniffi_trait_methods.to_string() %}
{%- let callable = to_string.callable %}
{%- let jni_method_name = to_string.jni_method_name %}
// The local Rust `Display`/`Debug` implementation.
override fun toString(): String {
    {% filter indent(4) %}{% include "CallableBody.kt" %}{% endfilter %}
}
{%- endif %}
{%- if let Some(eq) = uniffi_trait_methods.eq_eq %}
{%- let callable = eq.callable %}
{%- let jni_method_name = eq.jni_method_name %}
// The local Rust `Eq` implementation - only `eq` is used.
override fun equals(other: Any?): Boolean {
    if (other !is {{ type_name }}) return false
    {% filter indent(4) %}{% include "CallableBody.kt" %}{% endfilter %}
}
{%- endif %}
{%- if let Some(hash) = uniffi_trait_methods.hash_hash %}
{%- let callable = hash.callable %}
{%- let jni_method_name = hash.jni_method_name %}
// The local Rust `Hash` implementation
fun uniffiHashHash(): ULong {
    {% filter indent(4) %}{% include "CallableBody.kt" %}{% endfilter %}
}
override fun hashCode(): Int {
    return uniffiHashHash().toInt()
}
{%- endif %}
{%- if let Some(cmp) = uniffi_trait_methods.ord_cmp %}
{%- let callable = cmp.callable %}
{%- let jni_method_name = cmp.jni_method_name %}
fun uniffiOrdCmp(other: {{ type_name }}): Byte {
    {% filter indent(4) %}{% include "CallableBody.kt" %}{% endfilter %}
}
// The local Rust `Ord` implementation
override fun compareTo(other: {{ type_name }}): Int {
    return uniffiOrdCmp(other).toInt()
}
{%- endif %}
