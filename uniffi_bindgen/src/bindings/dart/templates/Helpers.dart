// An error type for FFI errors. These errors occur at the UniFFI level, not
// the library level.

class UniffiInternalError implements Exception {
  static const int bufferOverflow = 0;
  static const int incompleteData = 1;
  static const int unexpectedOptionalTag = 2;
  static const int unexpectedEnumCase = 3;
  static const int unexpectedNullPointer = 4;
  static const int unexpectedRustCallStatusCode = 5;
  static const int unexpectedRustCallError = 6;
  static const int unexpectedStaleHandle = 7;
  static const int rustPanic = 8;

  final int errorCode;
  final String? panicMessage;

  const UniffiInternalError(this.errorCode, this.panicMessage);

  static UniffiInternalError panicked(String message) {
    return UniffiInternalError(rustPanic, message);
  }

  @override
  String toString() {
    switch (errorCode) {
      case bufferOverflow:
        return "UniFfi::BufferOverflow";
      case incompleteData:
        return "UniFfi::IncompleteData";
      case unexpectedOptionalTag:
        return "UniFfi::UnexpectedOptionalTag";
      case unexpectedEnumCase:
        return "UniFfi::UnexpectedEnumCase";
      case unexpectedNullPointer:
        return "UniFfi::UnexpectedNullPointer";
      case unexpectedRustCallStatusCode:
        return "UniFfi::UnexpectedRustCallStatusCode";
      case unexpectedRustCallError:
        return "UniFfi::UnexpectedRustCallError";
      case unexpectedStaleHandle:
        return "UniFfi::UnexpectedStaleHandle";
      case rustPanic:
        return "UniFfi::rustPanic: $panicMessage";
      default:
        return "UniFfi::UnknownError: $errorCode"; 
    }
  }
}

const int CALL_SUCCESS = 0;
const int CALL_ERROR = 1;
const int CALL_PANIC = 2;

class RustCallStatus extends Struct {
  @Int8()
  external int code;
  external RustBuffer errorBuf;

  static Pointer<RustCallStatus> allocate({int count = 1}) =>
    calloc<RustCallStatus>(count * sizeOf<RustCallStatus>()).cast();
}

T noop<T>(T t) {
  return t;
}

T rustCall<T>(Api api, Function(Pointer<RustCallStatus>) callback) {
  var callStatus = RustCallStatus.allocate();
  final returnValue = callback(callStatus);

  switch (callStatus.ref.code) {
    case CALL_SUCCESS:
      calloc.free(callStatus);
      return returnValue;
    case CALL_ERROR:
      throw callStatus.ref.errorBuf;
    case CALL_PANIC:
      if (callStatus.ref.errorBuf.len > 0) {
        final message = {{ Type::String.borrow()|lift_fn }}(api, callStatus.ref.errorBuf);
        calloc.free(callStatus);
        throw UniffiInternalError.panicked(message);
      } else {
        calloc.free(callStatus);
        throw UniffiInternalError.panicked("Rust panic");
      }
    default:
      throw UniffiInternalError(callStatus.ref.code, null);
  }
}
