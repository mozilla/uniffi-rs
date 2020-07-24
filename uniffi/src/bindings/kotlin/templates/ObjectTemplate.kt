
class {{ obj.name()|class_name_kt }}(handle: Long) {
    private var handle: AtomicLong = AtomicLong(handle)
    {%- for cons in obj.constructors() %}
    constructor({% call kt::arg_list_decl(cons) -%}) :
        this({% call kt::to_rs_call(cons) %})
    {%- endfor %}

    // XXX TODO: destructors or equivalent.

    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}): {{ return_type|type_kt }} {
        val _retval = {% call kt::to_rs_call_with_prefix("this.handle.get()", meth) %}
        return {{ "_retval"|lift_kt(return_type) }}
    }
    
    {%- when None -%}
    fun {{ meth.name()|fn_name_kt }}({% call kt::arg_list_decl(meth) %}) =
        {% call kt::to_rs_call_with_prefix("this.handle.get()", meth) %}
    {% endmatch %}
    {% endfor %}
}