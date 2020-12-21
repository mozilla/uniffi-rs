public interface {{ obj.name()|class_name_kt }}Interface {
    {% for meth in obj.methods() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

class {{ obj.name()|class_name_kt }}(
    ptr: Pointer
) : FFIObject(AtomicReference(ptr)), {{ obj.name()|class_name_kt }}Interface {

    {%- for cons in obj.constructors() %}
    constructor({% call kt::arg_list_decl(cons) -%}) :
        this({% call kt::to_ffi_call(cons) %})
    {%- endfor %}

    /**
     * Disconnect the object from the underlying Rust object.
     *
     * It can be called more than once, but once called, interacting with the object 
     * causes an `IllegalStateException`.
     *
     * Clients **must** call this method once done with the object, or cause a memory leak.
     */
    override fun destroy() {
        try {
            callWithPointer {
                super.destroy() // poison the pointer so no-one else can use it before we tell rust.
                rustCall(InternalError.ByReference()) { err ->
                    _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(it, err)
                }
            }
        } catch (e: IllegalStateException) {
            // The user called this more than once. Better than less than once.
        }
    }

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}): {{ return_type|type_kt }} =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ "it"|lift_kt(return_type) }}
        }
    
    {%- when None -%}
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}) =
        callWithPointer {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %} 
        }
    {% endmatch %}
    {% endfor %}
}
