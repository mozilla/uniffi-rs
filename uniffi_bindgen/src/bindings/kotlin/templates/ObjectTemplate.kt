public interface {{ obj.name()|class_name_kt }}Interface {
    {% for meth in obj.methods() -%}
    {%- if ! meth.is_static() -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %})
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}: {{ return_type|type_kt -}}
    {%- else -%}
    {%- endmatch -%}
    {%- endif %}
    {% endfor %}
}

class {{ obj.name()|class_name_kt }}(
    handle: Long
) : FFIObject(AtomicLong(handle)), {{ obj.name()|class_name_kt }}Interface {

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
            callWithHandle {
                super.destroy() // poison the handle so no-one else can use it before we tell rust.
                rustCall(InternalError.ByReference()) { err ->
                    _UniFFILib.INSTANCE.{{ obj.ffi_object_free().name() }}(it, err)
                }
            }
        } catch (e: IllegalStateException) {
            // The user called this more than once. Better than less than once.
        }
    }

    {# // Instance methods #}
    {% for meth in obj.methods() -%}
    {%- if ! meth.is_static() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}): {{ return_type|type_kt }} =
        callWithHandle {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }.let {
            {{ "it"|lift_kt(return_type) }}
        }

    {%- when None -%}
    override fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_protocol(meth) %}) =
        callWithHandle {
            {%- call kt::to_ffi_call_with_prefix("it", meth) %}
        }
    {% endmatch %}
    {%- endif -%}
    {% endfor %}

    companion object {
        internal fun lift(handle: Long): {{ obj.name()|class_name_kt }} {
            return {{ obj.name()|class_name_kt }}(handle)
        }

        {# // Static methods, if any #}
        {% for meth in obj.methods() -%}
        {%- if meth.is_static() -%}
        {%- match meth.return_type() -%}

        {%- when Some with (return_type) -%}
        fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}): {{ return_type|type_kt }} {
            val _retval = {% call kt::to_ffi_call(meth) %}
            return {{ "_retval"|lift_kt(return_type) }}
        }

        {%- when None -%}
        fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}) =
            {% call kt::to_ffi_call(meth) %}
        {% endmatch %}
        {%- endif -%}
        {% endfor %}
    }
}
