public enum {{ e.name()|class_name_swift }}: Lowerable, Liftable {
    {% for variant in e.variants() %}
    case {{ variant|enum_variant_swift }}
    {% endfor %}

    static func lift(from buf: Reader) throws -> {{ e.name() }} {
        return try {{ e.name() }}.fromFFIValue(UInt32.lift(from: buf))
    }

    static func fromFFIValue(_ number: UInt32) throws -> {{ e.name() }} {
        switch number {
        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant|enum_variant_swift }}
        {% endfor %}
        default: throw InternalError.unexpectedEnumCase
        }
    }

    func lower(into buf: Writer) {
        self.toFFIValue().lower(into: buf)
    }

    func toFFIValue() -> UInt32 {
        switch self {
        {% for variant in e.variants() %}
        case .{{ variant|enum_variant_swift }}: return {{ loop.index }}
        {% endfor %}
        }
    }
}
