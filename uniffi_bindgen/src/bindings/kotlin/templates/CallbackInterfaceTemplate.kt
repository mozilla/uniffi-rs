{% import "macros.kt" as kt %}
public interface {{ cbi.nm() }} {
    {% for meth in cbi.methods() -%}
    fun {{ meth.nm() }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type.nm() -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

internal class {{ callback_interface_impl }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Long, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue {
        return {{ callback_internals }}.handleMap.callWithResult(handle) { cb ->
            when (method) {
                IDX_CALLBACK_FREE -> {{ callback_internals }}.drop(handle)
                {% for meth in cbi.methods() -%}
                {{ loop.index }} -> this.{{ self.invoke_method_name(meth) }}(cb, args)
                {% endfor %}
                // This should never happen, because an out of bounds method index won't
                // ever be used. Once we can catch errors, we should return an InternalException.
                // https://github.com/mozilla/uniffi-rs/issues/351
                else -> RustBuffer.ByValue()
            }
        }
    }

    {% for meth in cbi.methods() -%}
    private fun {{ self.invoke_method_name(meth) }}(kotlinCallbackInterface: {{ cbi.nm() }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            val buf = args.asByteBuffer() ?: throw InternalException("No ByteBuffer in RustBuffer; this is a Uniffi bug")
            kotlinCallbackInterface.{{ meth.nm() }}(
                    {% for arg in meth.arguments() -%}
                    {{ arg.type_().read() }}(buf)
                    {%- if !loop.last %}, {% endif %}
                    {% endfor -%}
                )
            {% else %}
            kotlinCallbackInterface.{{ meth.nm()}}()
            {% endif -%}

        {#- Packing up the return value into a RustBuffer #}
                {%- match meth.return_type() -%}
                {%- when Some with (return_type) -%}
                .let { rval ->
                    val rbuf = RustBufferBuilder()
                    {{ return_type.write() }}(rval, rbuf)
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

internal object {{ callback_internals }}: CallbackInternals<{{ cbi.nm() }}>(
    foreignCallback = {{ callback_interface_impl }}()
) {
    override fun register(lib: _UniFFILib) {
        rustCall() { status ->
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, status)
        }
    }
}

object {{ cbi.ffi_converter_name() }}: FFIConverter<{{ cbi.nm() }}, Long> {
    override fun lift(v: Long) = {{ callback_internals }}.handleMap.get(v)!!
    override fun lower(v: {{ cbi.nm() }}) = {{ callback_internals }}.handleMap.insert(v).also {
        assert({{ callback_internals }}.handleMap.get(it) === v) { "Handle map is not returning the object we just placed there. This is a bug in the HandleMap." }
    }
    override fun read(buf: ByteBuffer) = lift(buf.getLong())
    override fun write(v: {{ cbi.nm() }}, buf: RustBufferBuilder) = buf.putLong(lower(v))
}
