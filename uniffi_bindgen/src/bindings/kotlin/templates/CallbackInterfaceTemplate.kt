{% let type_name = cbi.name()|class_name_kt %}
public interface {{ type_name }} {
    {% for meth in cbi.methods() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

{% let canonical_type_name = cbi.type_().canonical_name()|class_name_kt %}
{% let callback_internals = format!("{}Internals", canonical_type_name) -%}
{% let callback_interface_impl = format!("{}FFI", canonical_type_name) -%}

internal class {{ callback_interface_impl }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Long, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue {
        return {{ callback_internals }}.handleMap.callWithResult(handle) { cb -> 
            when (method) {
                IDX_CALLBACK_FREE -> {{ callback_internals }}.drop(handle)
                {% for meth in cbi.methods() -%}
                {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt -%}
                {{ loop.index }} -> this.{{ method_name }}(cb, args)
                {% endfor %}
                // This should never happen, because an out of bounds method index won't
                // ever be used. Once we can catch errors, we should return an InternalError.
                // https://github.com/mozilla/uniffi-rs/issues/351
                else -> RustBuffer.ByValue()
            }
        }
    }

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt %}
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.has_arguments() -%}
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
                // TODO catch errors and report them back to Rust. 
                // https://github.com/mozilla/uniffi-rs/issues/351
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
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, err)
        }
    }
}