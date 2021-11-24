{% import "macros.swift" as swift %}
{%- let rec = self.inner() %}
public struct {{ rec|type_name }} {
    {%- for field in rec.fields() %}
    public var {{ field.name()|var_name }}: {{ field|type_name }}
    {%- endfor %}

    // Default memberwise initializers are never public by default, so we
    // declare one manually.
    public init({% call swift::field_list_decl(rec) %}) {
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name }} = {{ field.name()|var_name }}
        {%- endfor %}
    }
}

{% if ! self.contains_object_references() %}
extension {{ rec|type_name }}: Equatable, Hashable {
    public static func ==(lhs: {{ rec|type_name }}, rhs: {{ rec|type_name }}) -> Bool {
        {%- for field in rec.fields() %}
        if lhs.{{ field.name()|var_name }} != rhs.{{ field.name()|var_name }} {
            return false
        }
        {%- endfor %}
        return true
    }

    public func hash(into hasher: inout Hasher) {
        {%- for field in rec.fields() %}
        hasher.combine({{ field.name()|var_name }})
        {%- endfor %}
    }
}
{% endif %}

fileprivate extension {{ rec|type_name }} {
    static func read(from buf: Reader) throws -> {{ rec|type_name }} {
        return try {{ rec|type_name }}(
            {%- for field in rec.fields() %}
            {{ field.name()|var_name }}: {{ "buf"|read_var(field) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )
    }

    func write(into buf: Writer) {
        {%- for field in rec.fields() %}
        {{ "self.{}"|format(field.name())|write_var("buf", field) }}
        {%- endfor %}
    }
}

extension {{ rec|type_name }}: ViaFfiUsingByteBuffer, ViaFfi {}
