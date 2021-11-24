{% import "macros.swift" as swift %}
{%- let e = self.inner() %}
public enum {{ e|type_name }} {

    {% if e.is_flat() %}
    {% for variant in e.variants() %}
    // Simple error enums only carry a message
    case {{ variant.name()|class_name }}(message: String)
    {% endfor %}

    {%- else %}
    {% for variant in e.variants() %}
    case {{ variant.name()|class_name }}{% if variant.fields().len() > 0 %}({% call swift::field_list_decl(variant) %}){% endif -%}
    {% endfor %}

    {%- endif %}
}

extension {{ e|type_name }}: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> {{ e|type_name }} {
        let variant: Int32 = try buf.readInt()
        switch variant {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name }}(
            message: try {{ "buf"|read_var(Type::String) }}
        )
        {% endfor %}

       {% else %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name }}{% if variant.has_fields() -%}(
            {% for field in variant.fields() -%}
            {{ field.name()|var_name }}: try {{ "buf"|read_var(field) }}{% if loop.last %}{% else %},{% endif %}
            {% endfor -%}
        ){% endif -%}
        {% endfor %}

         {% endif -%}
        default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    fileprivate func write(into buf: Writer) {
        switch self {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case let .{{ variant.name()|class_name }}(message):
            buf.writeInt(Int32({{ loop.index }}))
            {{ "message"|write_var("buf", Type::String) }}
        {%- endfor %}

        {% else %}

        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|class_name }}({% for field in variant.fields() %}{{ field.name()|var_name }}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            buf.writeInt(Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field.name()|write_var("buf", field) }}
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|class_name }}:
            buf.writeInt(Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}

        {%- endif %}
        }
    }
}

{% if !self.contains_object_references() %}
extension {{ e|type_name }}: Equatable, Hashable {}
{% endif %}
extension {{ e|type_name }}: Error { }
