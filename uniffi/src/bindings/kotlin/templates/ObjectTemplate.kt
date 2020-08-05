class {{ obj.name()|class_name_kt }}(
    handle: Long
) : FFIObject(AtomicLong(handle)) {

    {%- for cons in obj.constructors() %}
    constructor({% call kt::arg_list_decl(cons) -%}) :
        this({% call kt::to_rs_call(cons) %})
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
            callWithHandle {
                super.destroy() // poison the handle so no-one else can use it before we tell rust.
                _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(it)
            }
        } catch (e: IllegalStateException) {
            // The user called this more than once. Better than less than once.
        }
    }

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}): {{ return_type|type_kt }} =
        callWithHandle {
            {% call kt::to_rs_call_with_prefix("it", meth) %} 
        }.let {
            {{ "it"|lift_kt(return_type) }}
        }
    
    {%- when None -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}) =
        callWithHandle {
            {% call kt::to_rs_call_with_prefix("it", meth) %} 
        }
    {% endmatch %}
    {% endfor %}
}