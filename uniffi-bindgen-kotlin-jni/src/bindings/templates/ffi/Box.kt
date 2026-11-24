{%- let type_name = box_.self_type.type_kt %}

@JvmName("{{ box_.self_type.lower_fn_kt() }}")
fun {{ box_.self_type.lower_fn_kt() }}(value: {{ type_name }}): kotlin.Long {
    val lowered = {{ box_.inner.lower_fn_kt() }}(value)
    return Scaffolding.{{ box_.jni_from_ffi_values_name() }}(
        {%- for (var, _) in box_.inner.ffi_values_kt("lowered") %}
        {{ var }},
        {%- endfor %}
    )
}

@JvmName("{{ box_.self_type.lift_fn_kt() }}")
fun {{ box_.self_type.lift_fn_kt() }}(value: kotlin.Long): {{ type_name }} {
    return Scaffolding.{{ box_.jni_into_inner_name() }}(value)
}

fun {{ box_.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    writeLong(buf, offset, {{ box_.self_type.lower_fn_kt() }}(value))
}

fun {{ box_.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    return {{ box_.self_type.lift_fn_kt() }}(readLong(buf, offset))
}

