public struct {{ rec.name()|class_name_swift }}:  ViaFfiUsingByteBuffer, ViaFfi, Equatable {
    {%- for field in rec.fields() %}
    let {{ field.name()|var_name_swift }}: {{ field.type_()|type_swift }}
    {%- endfor %}

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init(
        {%- for field in rec.fields() %}
        {{ field.name()|var_name_swift }}: {{ field.type_()|type_swift -}}
        {%- match field.default_value() %}
            {%- when Some with(literal) %} = {{ literal|literal_swift }}
            {%- else %}
        {%- endmatch -%}
        {% if !loop.last %}, {% endif %}
        {%- endfor %}
    ) {
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name_swift }} = {{ field.name()|var_name_swift }}
        {%- endfor %}
    }

    public static func ==(lhs: {{ rec.name()|class_name_swift }}, rhs: {{ rec.name()|class_name_swift }}) -> Bool {
        {%- for field in rec.fields() %}
        if lhs.{{ field.name()|var_name_swift }} != rhs.{{ field.name()|var_name_swift }} {
            return false
        }
        {%- endfor %}
        return true
    }

    static func read(from buf: Reader) throws -> {{ rec.name()|class_name_swift }} {
        return try {{ rec.name()|class_name_swift }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name_swift }}: {{ "buf"|read_swift(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    }

    func write(into buf: Writer) {
        {%- for field in rec.fields() %}
        {{ field.name()|var_name_swift }}.write(into: buf)
        {%- endfor %}
    }
}
