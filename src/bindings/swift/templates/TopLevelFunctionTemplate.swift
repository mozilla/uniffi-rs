{%- match func.return_type() -%}
{%- when Some with (return_type) %}

public func {{ func.name() }}(
    // TODO: More considered handling of labels (don't emit them
    // for single-argument functions; others?)
    {%- for arg in func.arguments() %}
    {{ arg.name() }}: {{ arg.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
) -> {{ return_type|decl_swift }} {
    let _retval = {{ func.ffi_func().name() }}(
        {%- for arg in func.arguments() %}
        {{ arg.name()|lower_swift(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    )
    return try! {{ "_retval"|lift_swift(return_type) }}
}

{% when None -%}

public func {{ func.name() }}(
    {%- for arg in func.arguments() %}
    {{ arg.name() }}: {{ arg.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
    {%- endfor %}
) {
    {{ func.ffi_func().name() }}(
        {%- for arg in func.arguments() %}
        {{ arg.name()|lower_swift(arg.type_()) }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    )
}

{%- endmatch %}