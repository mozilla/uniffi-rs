{% let type_name = obj.name()|class_name_kt %}
public interface {{ type_name }} {
    {% for meth in obj.methods() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

{% let canonical_type_name = format!("CallbackInterface{}", type_name) %}
{% let callback_internals = format!("{}Internals", canonical_type_name) -%}
{% let callback_interface_impl = format!("{}FFI", canonical_type_name) -%}

internal class {{ callback_interface_impl }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Long, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue {
        return {{ callback_internals }}.handleMap.callWithResult(handle) { cb -> 
            when (method) {
                IDX_CALLBACK_FREE -> {{ callback_internals }}.drop(handle)
                {% for meth in obj.methods() -%}
                {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt -%}
                {{ loop.index }} -> this.{{ method_name }}(cb, args)
                {% endfor %}
                else -> RustBuffer.ByValue()
            }
        }
    }

    {% for meth in obj.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt %}
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            val buf = args.asByteBuffer() ?: throw InternalError("No ByteBuffer in RustBuffer; this is a Uniffi bug")
            kotlinCallbackInterface.{{ meth.name()|fn_name_kt }}(
                    {% for arg in meth.arguments() -%}
                    {{ "buf"|read_kt(arg.type_()) }}
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            {% else %}
            kotlinCallbackInterface.{{ meth.name()|fn_name_kt }}()
            {% endif -%}

        {#- Packing up the return value into a RustBuffer #}
                {%- match meth.return_type() -%}
                {%- when Some with (return_type) -%}
                .let { rval -> 
                    val rbuf = RustBufferBuilder()
                    {{ "rval"|write_kt("rbuf", return_type) }} 
                    rbuf.finalize()
                }
                {%- else -%}
                .let { RustBuffer.ByValue() }
                {% endmatch -%}
        } finally {
            RustBuffer.free(args)
        }

    {% endfor %}
}

internal object {{ callback_internals }}: CallbackInternals<{{ type_name }}>(
    foreignCallback = {{ callback_interface_impl }}()
) {
    override fun register(lib: _UniFFILib) {
        rustCall(InternalError.ByReference()) { err ->
            lib.{{ obj.ffi_init_callback().name() }}(this.foreignCallback, err)
        }
    }
}