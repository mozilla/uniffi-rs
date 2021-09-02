{% import "macros.swift" as swift %}
{%- let e = self.inner() %}
public enum {{ e.name()|class_name_swift }} {

    {% if e.is_flat() %}
    {% for variant in e.variants() %}
    // Simple error enums only carry a message
    case {{ variant.name()|class_name_swift }}(message: String)
    {% endfor %}

    {%- else %}
    {% for variant in e.variants() %}
    case {{ variant.name()|class_name_swift }}{% if variant.fields().len() > 0 %}({% call swift::field_list_decl(variant) %}){% endif -%}
    {% endfor %}

    {%- endif %}
}

extension {{ e.name()|class_name_swift }}: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> {{ e.name()|class_name_swift }} {
        let variant: Int32 = try buf.readInt()
        switch variant {

        {% if e.is_flat() %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name_swift }}(
            message: try String.read(from: buf)
        )
        {% endfor %}

       {% else %}

        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|class_name_swift }}{% if variant.has_fields() -%}(
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}: try {{ "buf"|read_swift(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
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
        case let .{{ variant.name()|class_name_swift }}(message):
            buf.writeInt(Int32({{ loop.index }}))
            message.write(into: buf)
        {%- endfor %}

        {% else %}

        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|class_name_swift }}({% for field in variant.fields() %}{{ field.name()|var_name_swift }}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            buf.writeInt(Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}.write(into: buf)
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|class_name_swift }}:
            buf.writeInt(Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}

        {%- endif %}
        }
    }
}

{% if !self.contains_object_references() %}
extension {{ e.name()|class_name_swift }}: Equatable, Hashable {}
{% endif %}
extension {{ e.name()|class_name_swift }}: Error { }
