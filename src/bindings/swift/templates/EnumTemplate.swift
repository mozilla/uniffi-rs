public enum {{ e.name() }}: Lowerable, Liftable {
    {% for value in e.values() %}
    case {{ value|decl_enum_variant_swift }}
    {% endfor %}

    static func lift(from buf: Reader) throws -> {{ e.name() }} {
        return try {{ e.name() }}.fromFFIValue(UInt32.lift(from: buf))
    }

    static func fromFFIValue(_ number: UInt32) throws -> {{ e.name() }} {
        switch number {
        {% for value in e.values() %}
        case {{ loop.index }}: return .{{ value|decl_enum_variant_swift }}
        {% endfor %}
        default: throw InternalError.unexpectedEnumCase
        }
    }

    func lower(into buf: Writer) {
        self.toFFIValue().lower(into: buf)
    }

    func toFFIValue() -> UInt32 {
        switch self {
        {% for value in e.values() %}
        case .{{ value|decl_enum_variant_swift }}: return {{ loop.index }}
        {% endfor %}
        }
    }
}
