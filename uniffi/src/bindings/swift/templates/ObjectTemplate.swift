public class {{ obj.name() }} {
    private let handle: UInt64

    {%- for cons in obj.constructors() %}
    public init(
        {%- for arg in cons.arguments() %}
        {{ arg.name() }}: {{ arg.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    ) {
        self.handle = {{ cons.ffi_func().name() }}(
            {%- for arg in cons.arguments() %}
            {{ arg.name()|lower_swift(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        );
    }
    {%- endfor %}

    // XXX TODO: destructors or equivalent.

    {%- for meth in obj.methods() %}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %}

    public func {{ meth.name() }}(
        {% for arg in meth.arguments() %}
        {{ arg.name() }}: {{ arg.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
        {% endfor %}
    ) -> {{ return_type|decl_swift }} {
        let _retval = {{ meth.ffi_func().name() }}(
            self.handle{% if meth.arguments().len() > 0 %},{% endif %}
            {%- for arg in meth.arguments() %}
            {{ arg.name()|lower_swift(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
        return try! {{ "_retval"|lift_swift(return_type) }}
    }

    {% when None -%}

    public func {{ meth.name() }}(
        {% for arg in meth.arguments() %}
        {{ arg.name() }}: {{ arg.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
        {% endfor %}
    ) {
        {{ meth.ffi_func().name() }}(
            self.handle{% if meth.arguments().len() > 0 %},{% endif %}
            {%- for arg in meth.arguments() %}
            {{ arg.name()|lower_swift(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    }

    {%- endmatch %}
    {%- endfor %}
}