{#
// In here we define conversions between a native reference to Swift errors
// We use the RustError protocol to define the requirements. Any implementers of the protocol
// Can be generated from a NativeRustError.

#}

fileprivate protocol RustError: LocalizedError {
    static func fromConsuming(_ rustError: NativeRustError) throws -> Self?
}

// An error type for FFI errors. These errors occur at the UniFFI level, not
// the library level.
fileprivate enum UniffiInternalError: RustError {
    case bufferOverflow
    case incompleteData
    case unexpectedOptionalTag
    case unexpectedEnumCase
    case unexpectedNullPointer
    case emptyResult
    case unknown(_ message: String)

    public var errorDescription: String? {
        switch self {
        case .bufferOverflow: return "Reading the requested value would read past the end of the buffer"
        case .incompleteData: return "The buffer still has data after lifting its containing value"
        case .unexpectedOptionalTag: return "Unexpected optional tag; should be 0 or 1"
        case .unexpectedEnumCase: return "Raw enum value doesn't match any cases"
        case .unexpectedNullPointer: return "Raw pointer value was null"
        case .emptyResult: return "Unexpected nil returned from FFI function"
        case let .unknown(message): return "FFI function returned unknown error: \(message)"
        }
    }

    fileprivate static func fromConsuming(_ rustError: NativeRustError) throws -> Self? {
        let message = rustError.message
        defer {
            if message != nil {
                try! rustCall(UniffiInternalError.unknown("UniffiInternalError.fromConsuming")) { err in
                    {{ ci.ffi_string_free().name() }}(message!, err)
                }
            }
        }
        switch rustError.code {
        case 0: return nil
        default: return .unknown(String(cString: message!))
        }
    }
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
    fileprivate static func fromConsuming(_ rustError: NativeRustError) throws -> Self? {
        let message = rustError.message
        defer {
            if message != nil {
                try! rustCall(UniffiInternalError.unknown("{{e.name()}}.fromConsuming")) { err in
                    {{ ci.ffi_string_free().name() }}(message!, err)
                }
            }
        }
        switch rustError.code {
            case 0:
                return nil
            {% for value in e.values() %}
            case {{loop.index}}:
                return .{{value}}(message: String(cString: message!))
            {% endfor %}
            default:
                return nil
        }
    }
}
{% endfor %}

private func rustCall<T, E: RustError>(_ err: E, _ cb: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T {
    return try unwrap(err) { native_err in
        return try cb(native_err)
    }
}

private func nullableRustCall<T, E: RustError>(_ err: E, _ cb: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T? {
    return try tryUnwrap(err) { native_err in
        return try cb(native_err)
    }
}

@discardableResult
private func unwrap<T, E: RustError>(_ err: E, _ callback: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T {
    guard let result = try tryUnwrap(err, callback) else {
        throw UniffiInternalError.emptyResult
    }
    return result
}

@discardableResult
private func tryUnwrap<T, E: RustError>(_ err: E, _ callback: (UnsafeMutablePointer<NativeRustError>) throws -> T?) throws -> T? {
    var native_err = NativeRustError(code: 0, message: nil)
    let returnedVal = try callback(&native_err)
    if let retErr = try E.fromConsuming(native_err) {
        throw retErr
    }
    return returnedVal
}
