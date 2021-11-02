{% import "macros.kt" as kt %}
{%- let cbi = self.inner() %}
{%- let type_name = cbi|type_kt %}
{%- let canonical_type_name = cbi|canonical_name %}
{%- let ffi_converter = format!("FfiConverter{}", canonical_type_name) %}
{%- let foreign_callback = format!("ForeignCallback{}", canonical_type_name) %}

// Declaration and FfiConverters for {{ type_name }} Callback Interface

public interface {{ type_name }} {
    {% for meth in cbi.methods() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

// The ForeignCallback that is passed to Rust.
internal class {{ foreign_callback }} : ForeignCallback {
    @Suppress("TooGenericExceptionCaught")
    override fun invoke(handle: Handle, method: Int, args: RustBuffer.ByValue): RustBuffer.ByValue {
        val cb = {{ ffi_converter }}.lift(handle) ?: throw InternalException("No callback in handlemap; this is a Uniffi bug")
        return when (method) {
            IDX_CALLBACK_FREE -> {{ ffi_converter }}.drop(handle)
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

    {% for meth in cbi.methods() -%}
    {% let method_name = format!("invoke_{}", meth.name())|fn_name_kt %}
    private fun {{ method_name }}(kotlinCallbackInterface: {{ type_name }}, args: RustBuffer.ByValue): RustBuffer.ByValue =
        try {
        {#- Unpacking args from the RustBuffer #}
            {%- if meth.arguments().len() != 0 -%}
            {#- Calling the concrete callback object #}
            val buf = args.asByteBuffer() ?: throw InternalException("No ByteBuffer in RustBuffer; this is a Uniffi bug")
            kotlinCallbackInterface.{{ meth.name()|fn_name_kt }}(
                    {% for arg in meth.arguments() -%}
                    {{ "buf"|read_kt(arg) }}
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

// The ffiConverter which transforms the Callbacks in to Handles to pass to Rust.
internal object {{ ffi_converter }}: FfiConverterCallbackInterface<{{ type_name }}>(
    foreignCallback = {{ foreign_callback }}()
) {
    override fun register(lib: _UniFFILib) {
        rustCall() { status ->
            lib.{{ cbi.ffi_init_callback().name() }}(this.foreignCallback, status)
        }
    }
}
