{#
// In here we define conversions between a native reference to Swift errors
// We use the RustError protocol to define the requirements. Any implementers of the protocol
// Can be generated from a NativeRustError. 

#}

protocol RustError: LocalizedError {
    static func fromConsuming(_ rustError: NativeRustError) throws -> Self?
}

{% for e in ci.iter_error_definitions() %}
public enum {{e.name()}}: RustError {
    case NoError
    {% for value in e.values() %}
    case {{value}}(message: String)
    {% endfor %}


    /// Our implementation of the localizedError protocol
    public var errorDescription: String? {
        switch self {
        {% for value in e.values() %}
        case let .{{value}}(message):
            return "{{e.name()}}.{{value}}: \(message)"
        {% endfor %}
        default:
            return nil
        }
    }

    // The name is attempting to indicate that we free message if it
    // existed, and that it's a very bad idea to touch it after you call this
    // function
    static func fromConsuming(_ rustError: NativeRustError) throws -> Self? {
        let message = rustError.message
        switch rustError.code {
            case 0:
                return nil
            {% for value in e.values() %}
            case {{loop.index}}:
                return .{{value}}(message: try String.lift(message!))
            {% endfor %}
            default:
                return nil
        }
    }
}
{% endfor %}

func rustCall<T, E: RustError>(_ err: E, _ cb: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T {
    return try unwrap(err) { native_err in
        return try cb(native_err)
    }
}

func nullableRustCall<T, E: RustError>(_ err: E, _ cb: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T? {
    return try tryUnwrap(err) { native_err in
        return try cb(native_err)
    }
}

@discardableResult
 func unwrap<T, E: RustError>(_ err: E, _ callback: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T {
    guard let result = try tryUnwrap(err,callback) else {
        throw InternalError.emptyResult
    }
    return result
}

@discardableResult
func tryUnwrap<T, E: RustError>(_ err: E, _ callback: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T? {
    var native_err = NativeRustError(code: 0, message: nil)
    let returnedVal = try callback(&native_err)
    if let retErr = try type(of: err).fromConsuming(native_err) {
        throw retErr
    }
    return returnedVal
}
