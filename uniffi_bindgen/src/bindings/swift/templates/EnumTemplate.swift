
// Note that we don't yet support `indirect` for enums.
// See https://github.com/mozilla/uniffi-rs/issues/396 for further discussion.
public enum {{ e.name()|class_name_swift }} {
    {% for variant in e.variants() %}
    case {{ variant.name()|enum_variant_swift }}{% if variant.fields().len() > 0 %}({% call swift::field_list_decl(variant) %}){% endif -%}
    {% endfor %}
}

extension {{ e.name()|class_name_swift }}: ViaFfiUsingByteBuffer, ViaFfi {
    fileprivate static func read(from buf: Reader) throws -> {{ e.name()|class_name_swift }} {
        let variant: Int32 = try buf.readInt()
        switch variant {
        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant.name()|enum_variant_swift }}{% if variant.has_fields() -%}(
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}: try {{ "buf"|read_swift(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {% endfor -%}
        ){% endif -%}
        {% endfor %}
        default: throw UniffiInternalError.unexpectedEnumCase
        }
    }

    fileprivate func write(into buf: Writer) {
        switch self {
        {% for variant in e.variants() %}
        {% if variant.has_fields() %}
        case let .{{ variant.name()|enum_variant_swift }}({% for field in variant.fields() %}{{ field.name()|var_name_swift }}{%- if loop.last -%}{%- else -%},{%- endif -%}{% endfor %}):
            buf.writeInt(Int32({{ loop.index }}))
            {% for field in variant.fields() -%}
            {{ field.name()|var_name_swift }}.write(into: buf)
            {% endfor -%}
        {% else %}
        case .{{ variant.name()|enum_variant_swift }}:
            buf.writeInt(Int32({{ loop.index }}))
        {% endif %}
        {%- endfor %}
        }
    }
}

{% if ! e.contains_object_references(ci) %}
extension {{ e.name()|class_name_swift }}: Equatable, Hashable {}
{% endif %}
