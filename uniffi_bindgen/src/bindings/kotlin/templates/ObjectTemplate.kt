{% import "macros.kt" as kt %}
public interface {{ obj.nm() }}Interface {
    {% for meth in obj.methods() -%}
    fun {{ meth.nm() }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type.nm() -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

class {{ obj.nm() }}(
    pointer: Pointer
) : FFIObject(pointer), {{ obj.nm() }}Interface {

    {%- match obj.primary_constructor() %}
    {%- when Some with (cons) %}
    constructor({% call kt::arg_list_decl(cons) -%}) :
        this({% call kt::to_ffi_call(cons) %})
    {%- when None %}
    {%- endmatch %}

    /**
     * Disconnect the object from the underlying Rust object.
     *
     * It can be called more than once, but once called, interacting with the object
     * causes an `IllegalStateException`.
     *
     * Clients **must** call this method once done with the object, or cause a memory leak.
     */
    override protected fun freeRustArcPtr() {
        rustCall() { status ->
            _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(this.pointer, status)
        }
    }

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    override fun {{ meth.nm() }}({% call kt::arg_list_protocol(meth) %}): {{ return_type.nm() }} =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ return_type.lift() }}(it)
        }

    {%- when None -%}
    override fun {{ meth.nm() }}({% call kt::arg_list_protocol(meth) %}) =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }
    {% endmatch %}
    {% endfor %}

    {% if obj.has_alternate_constructor() -%}
    companion object {
        {% for cons in obj.alternate_constructors() -%}
        fun {{ cons.nm() }}({% call kt::arg_list_decl(cons) %}): {{ obj.nm() }} =
            {{ obj.nm() }}({% call kt::to_ffi_call(cons) %})
        {% endfor %}
    }
    {% endif %}
}

object {{ obj.ffi_converter_name() }}: FFIConverter<{{ obj.nm() }}, Pointer> {
    override fun lower(v: {{ obj.nm() }}): Pointer = v.callWithPointer { it }

    override fun write(v: {{ obj.nm() }}, buf: RustBufferBuilder) {
        // The Rust code always expects pointers written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(Pointer.nativeValue(lower(v)))
    }

    override fun lift(v: Pointer): {{ obj.nm() }} {
        return {{ obj.nm() }}(v)
    }

    override fun read(buf: ByteBuffer): {{ obj.nm() }} {
        // The Rust code always writes pointers as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(Pointer(buf.getLong()))
    }
}
