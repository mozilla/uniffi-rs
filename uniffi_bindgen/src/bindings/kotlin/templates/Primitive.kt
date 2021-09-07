object {{ ffi_converter_name }}: FFIConverter<{{ type_name }}, {{ ffi_name }}> {
    override fun lift(v: {{ ffi_name }} ): {{ type_name }} {
        return {{ lift_expr }}
    }

    override fun read(buf: ByteBuffer): {{ type_name }} {
        return {{ read_expr }}
    }

    override fun lower(v: {{ type_name }}): {{ ffi_name }}  {
        return {{ lower_expr }}
    }

    override fun write(v: {{ type_name }}, buf: RustBufferBuilder) {
        {{ write_expr }}
    }
}
