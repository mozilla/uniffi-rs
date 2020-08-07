public struct {{ rec.name()|class_name_swift }}: Lowerable, Liftable, Serializable, Equatable {
    {%- for field in rec.fields() %}
    let {{ field.name()|var_name_swift }}: {{ field.type_()|decl_swift }}
    {%- endfor %}

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        {%- for field in rec.fields() %}
        {{ field.name()|var_name_swift }}: {{ field.type_()|decl_swift }}{% if loop.last %}{% else %},{% endif %}
        {%- endfor %}
    ) {
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name_swift }} = {{ field.name()|var_name_swift }}
        {%- endfor %}
    }

    public static func ==(lhs: {{ rec.name() }}, rhs: {{ rec.name() }}) -> Bool {
        {%- for field in rec.fields() %}
        if lhs.{{ field.name()|var_name_swift }} != rhs.{{ field.name()|var_name_swift }} {
            return false
        }
        {%- endfor %}
        return true
    }

    static func lift(from buf: Reader) throws -> {{ rec.name() }} {
        return try {{ rec.name() }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name_swift }}: {{ "buf"|lift_from_swift(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    }

    func lower(into buf: Writer) {
        {%- for field in rec.fields() %}
        {{ field.name()|var_name_swift }}.lower(into: buf)
        {%- endfor %}
    }
}