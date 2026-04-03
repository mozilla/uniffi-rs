{%- let type_name = cls.self_type.type_kt %}
{%- let object_to_handle = "objectToHandle{}"|format(cls.self_type.id) %}
{%- let handle_to_object = "handleToObject{}"|format(cls.self_type.id) %}
{%- let lower_object_ref = "lowerObjectRef{}"|format(cls.self_type.id) %}

fun {{ object_to_handle }}(obj: {{ type_name }}): kotlin.Long {
    return obj.uniffiCloneHandle()
}

fun {{ handle_to_object }}(handle: kotlin.Long): {{ type_name }} {
    return {{ type_name }}(uniffi.WithHandle, handle)
}

@JvmName("{{ cls.self_type.lower_fn_kt() }}")
fun {{ cls.self_type.lower_fn_kt() }}(value: {{ type_name }}): kotlin.Long {
    return {{ object_to_handle }}(value)
}

@JvmName("{{ cls.self_type.lift_fn_kt() }}")
fun {{ cls.self_type.lift_fn_kt() }}(handle: kotlin.Long): {{ type_name }} {
    return {{ handle_to_object }}(handle)
}

fun {{ cls.self_type.write_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int, value: {{ type_name }}) {
    writeLong(buf, offset, {{ object_to_handle }}(value))
}

fun {{ cls.self_type.read_fn_kt() }}(buf: java.nio.ByteBuffer, offset: kotlin.Int): {{ type_name }} {
    return {{ handle_to_object }}(readLong(buf, offset))
}


@JvmName("{{ lower_object_ref }}")
fun {{ lower_object_ref }}(value: {{ type_name }}): kotlin.Long {
    return value.uniffiHandle
}

