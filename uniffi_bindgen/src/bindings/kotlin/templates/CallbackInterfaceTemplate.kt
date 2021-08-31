{% import "macros.kt" as kt %}
{%- let cbi = self.inner() %}
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
    {% call kt::unsigned_types_annotation(self) %}
    override fun invoke(handle: Long, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue {
        return {{ callback_internals }}.handleMap.callWithResult(handle) { cb ->
            when (method) {
                IDX_CALLBACK_FREE -> {{ callback_internals }}.drop(handle)
                {% for meth in cbi.methods() -%}
                {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt -%}
                {{ loop.index }} -> this.{{ method_name }}(cb, args)
                {% endfor %}
                // This should never happen, because an out of bounds method index won't
                // ever be used. Once we can catch errors, we should return an InternalException.
                // https://github.com/mozilla/uniffi-rs/issues/351
                else -> RustBuffer.ByValue()
            }
        }
    }

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt %}
    {% call kt::unsigned_types_annotation(self) %}
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            val buf = args.asByteBuffer() ?: throw InternalException("No ByteBuffer in RustBuffer; this is a Uniffi bug")
            kotlinCallbackInterface.{{ meth.name()|fn_name_kt }}(
                    {% for arg in meth.arguments() -%}
                    {{ arg.type_()|ffi_converter_name }}.read(buf)
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
                    {{ return_type|ffi_converter_name }}.write(rval, rbuf::write)
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
        rustCall() { status ->
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, status)
        }
    }
}

{% let type_ = cbi.type_() %}

object {{ type_|ffi_converter_name }}: FFIConverter<{{ type_name }}, Long> {
    override fun lift(v: Long) = {{ callback_internals }}.handleMap.get(v)!!
    override fun lower(v: {{ type_name }}) = {{ callback_internals }}.handleMap.insert(v).also {
        assert({{ callback_internals }}.handleMap.get(it) === v) { "Handle map is not returning the object we just placed there. This is a bug in the HandleMap." }
    }
    override fun read(buf: ByteBuffer) = lift(buf.getLong())
    override fun write(v: {{ type_name }}, bufferWrite: BufferWriteFunc) = putLong(lower(v), bufferWrite)
}

