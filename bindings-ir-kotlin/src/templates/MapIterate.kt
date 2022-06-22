{%- set tmp_var = temp_var_name() -%}
for ({{ tmp_var }} in {{ map }}.entries.iterator()) {
    val {{ key_var }} = {{ tmp_var }}.key;
    val {{ val_var }} = {{ tmp_var }}.value;
    {{ block }}
}
