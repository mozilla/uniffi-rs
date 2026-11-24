{%- let type_name = cls.self_type.type_kt %}
{%- let impl_class_name = "{}.{}"|format(cls.package_name, cls.name_kt()) %}
{%- let object_to_handle = "objectToHandle{}"|format(cls.self_type.id) %}
{%- let handle_to_object = "handleToObject{}"|format(cls.self_type.id) %}
{%- let lower_object_ref = "lowerObjectRef{}"|format(cls.self_type.id) %}
{%- let lower_object_receiver_ref = "lowerObjectReceiverRef{}"|format(cls.self_type.id) %}

{%- if !cls.imp.has_callback_interface() %}
fun {{ object_to_handle }}(obj: {{ type_name }}): kotlin.Long {
    return obj.uniffiCloneHandle()
}

fun {{ handle_to_object }}(handle: kotlin.Long): {{ type_name }} {
    return {{ impl_class_name }}(uniffi.WithHandle, handle)
}

@JvmName("{{ lower_object_ref }}")
fun {{ lower_object_ref }}(value: {{ type_name }}): kotlin.Long {
    return value.uniffiHandle
}

{%- else %}
fun {{ object_to_handle }}(obj: {{ type_name }}): kotlin.Long {
    if (obj is {{ impl_class_name }}) {
        // Rust-implemented interface
        return obj.uniffiCloneHandle()
    } else {
        // Kotlin-implemented interface
        return {{ cls.handle_map_kt() }}.insert(obj)
    }
}

fun {{ handle_to_object }}(handle: kotlin.Long): {{ type_name }} {
    if ((handle and 1L) == 0L) {
        // Rust-implemented interface
        return {{ impl_class_name }}(uniffi.WithHandle, handle)
    } else {
        // Kotlin-implemented interface
        return {{ cls.handle_map_kt() }}.remove(handle)
    }
}

@JvmName("{{ lower_object_receiver_ref }}")
fun {{ lower_object_receiver_ref }}(value: {{ impl_class_name }}): kotlin.Long {
    return value.uniffiHandle
}
{%- endif %}

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
