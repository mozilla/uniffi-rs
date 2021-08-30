{% import "macros.kt" as kt %}
{%- let obj = self.inner() %}
{% call kt::unsigned_types_annotation(self) %}
public interface {{ obj.name()|class_name_kt }}Interface {
    {% for meth in obj.methods() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

{% call kt::unsigned_types_annotation(self) %}
class {{ obj.name()|class_name_kt }}(
    pointer: Pointer
) : FFIObject(pointer), {{ obj.name()|class_name_kt }}Interface {

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
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}): {{ return_type|type_kt }} =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ return_type|ffi_converter_name }}.lift(it)
        }

    {%- when None -%}
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}) =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }
    {% endmatch %}
    {% endfor %}

    companion object {
        {% for cons in obj.alternate_constructors() -%}
        fun {{ cons.name()|fn_name_kt }}({% call kt::arg_list_decl(cons) %}): {{ obj.name()|class_name_kt }} =
            {{ obj.name()|class_name_kt }}({% call kt::to_ffi_call(cons) %})
        {% endfor %}
    }
}

{% let type_ = obj.type_() %}

{% call kt::unsigned_types_annotation(self) %}
object {{ type_|ffi_converter_name }} {
    internal fun lift(ptr: Pointer) = {{ obj.name()|class_name_kt }}(ptr)

    internal fun lower(obj: {{ obj.name()|class_name_kt }}): Pointer = obj.pointer

    internal fun read(buf: ByteBuffer): {{ obj.name()|class_name_kt }} {
        // The Rust code always writes pointers as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(Pointer(buf.getLong()))
    }

    internal fun write(obj: {{ obj.name()|class_name_kt }}, buf: RustBufferBuilder) {
        // The Rust code always expects pointers written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(Pointer.nativeValue(lower(obj)))
    }
}

