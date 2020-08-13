public enum {{ e.name()|class_name_swift }}: ViaFfi {
    {% for variant in e.variants() %}
    case {{ variant|enum_variant_swift }}
    {% endfor %}

    static func read(from buf: Reader) throws -> {{ e.name()|class_name_swift }} {
        return try {{ e.name()|class_name_swift }}.lift(UInt32.read(from: buf))
    }

    static func lift(_ number: UInt32) throws -> {{ e.name()|class_name_swift }} {
        switch number {
        {% for variant in e.variants() %}
        case {{ loop.index }}: return .{{ variant|enum_variant_swift }}
        {% endfor %}
        default: throw InternalError.unexpectedEnumCase
        }
    }

    func write(into buf: Writer) {
        self.lower().write(into: buf)
    }

    func lower() -> UInt32 {
        switch self {
        {% for variant in e.variants() %}
        case .{{ variant|enum_variant_swift }}: return {{ loop.index }}
        {% endfor %}
        }
    }
}
