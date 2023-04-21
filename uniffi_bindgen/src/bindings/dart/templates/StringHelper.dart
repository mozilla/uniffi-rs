typedef StringLowered = RustBuffer;
typedef StringLifted = String;

typedef StringFfi = RustBuffer;

class FfiConverterString {
  static StringLifted lift(Api api, StringLowered value) {
    final codeUnits = value.data.cast<Uint8>();
    return utf8.decode(codeUnits.asTypedList(value.len));
  }

  static RustBuffer lower(Api api, String value) {
    final units = utf8.encode(value);
    final Pointer<Uint8> result = calloc<Uint8>(units.length);
    final Uint8List nativeString = result.asTypedList(units.length);
    nativeString.setAll(0, units);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.data = result.cast();
    bytes.ref.len = units.length;
    return RustBuffer.fromBytes(api, bytes);
  }
}
