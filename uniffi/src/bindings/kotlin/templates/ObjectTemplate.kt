class {{ obj.name()|class_name_kt }}(handle: Long) {
    private var handle: AtomicLong = AtomicLong(handle)
    {%- for cons in obj.constructors() %}
    constructor({% for arg in cons.arguments() %}{{ arg.name() }}: {{ arg.type_()|type_kt }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}) :
        this(
            _UniFFILib.INSTANCE.{{ cons.ffi_func().name() }}(
                {%- for arg in cons.arguments() %}
                {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
                {%- endfor %}
            )
        )
    {%- endfor %}

    // XXX TODO: destructors or equivalent.

    {%- for meth in obj.methods() %}
    fun {{ meth.name()|fn_name_kt }}(
        {% for arg in meth.arguments() %}
        {{ arg.name() }}: {{ arg.type_()|type_kt }}{% if loop.last %}{% else %}, {% endif %}
        {% endfor %}
    ): {% match meth.return_type() %}{% when Some with (type_) %}{{ type_|type_kt }}{% when None %}Unit{% endmatch %} {
        val _retval = _UniFFILib.INSTANCE.{{ meth.ffi_func().name() }}(
            this.handle.get(){% if meth.arguments().len() > 0 %},{% endif %}
            {%- for arg in meth.arguments() %}
            {{ arg.name()|lower_kt(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        {% match meth.return_type() %}{% when Some with (return_type) %}return {{ "_retval"|lift_kt(return_type) }}{% else %}return _retval{% endmatch %}
    }
    {%- endfor %}
}