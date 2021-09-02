internal fun {{ type_name }}.Companion.lift(v: {{ ffi_name }} ): {{ type_name }} {
    return {{ lift_expr }}
}

internal fun {{ type_name }}.Companion.read(buf: ByteBuffer): {{ type_name }} {
    return {{ read_expr }}
}

internal fun {{ type_name }}.lower(): {{ ffi_name }}  {
    return {{ lower_expr }}
}

internal fun {{ type_name }}.write(buf: RustBufferBuilder) {
    {{ write_expr }}
}
