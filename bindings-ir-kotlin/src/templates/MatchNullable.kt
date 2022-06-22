{%- set tmp_var = temp_var_name() -%}
val {{ tmp_var }} = {{ value }}
if ({{ tmp_var }} == null) {
    {{ null_arm.block }}
} else {
    val {{ some_arm.var }} = {{ tmp_var }}
    {{ some_arm.block }}
}
