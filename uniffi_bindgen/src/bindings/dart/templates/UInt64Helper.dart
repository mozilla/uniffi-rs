typedef Uint64Ffi = Uint64;
typedef Uint64Lowered = Uint64;
typedef Uint64Lifted = int;
typedef Uint64DartFfi = int;

class FfiConverterUint64 {
    static Uint64Lowered lift(Uint64Lowered value) {
        return value;
    }

    static Uint64Lifted lower(Uint64Lifted value) {
        return value;
    }
}
