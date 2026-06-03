import uniffi.uniffi_bindgen_tests.*

assert(roundtripOptionU8(67.toUByte()) == 67.toUByte());
assert(roundtripOptionU8(null) == null);
assert(roundtripOptionI8(67.toByte()) == 67.toByte());
assert(roundtripOptionI8(null) == null);
assert(roundtripOptionU16(67.toUShort()) == 67.toUShort());
assert(roundtripOptionU16(null) == null);
assert(roundtripOptionI16(67.toShort()) == 67.toShort());
assert(roundtripOptionI16(null) == null);
assert(roundtripOptionU32(67u) == 67u);
assert(roundtripOptionU32(null) == null);
assert(roundtripOptionI32(67) == 67);
assert(roundtripOptionI32(null) == null);
assert(roundtripOptionU64(67uL) == 67uL);
assert(roundtripOptionU64(null) == null);
assert(roundtripOptionI64(67L) == 67L);
assert(roundtripOptionI64(null) == null);
assert(roundtripOptionF32(67.0f) == 67.0f);
assert(roundtripOptionF32(null) == null);
assert(roundtripOptionF64(67.0) == 67.0);
assert(roundtripOptionF64(null) == null);
assert(roundtripOptionBool(true) == true);
assert(roundtripOptionBool(null) == null);
assert(roundtripOptionString("test-string") == "test-string");
assert(roundtripOptionString(null) == null);
assert(roundtripOptionRec(OptionsRec(67.toUByte())) == OptionsRec(67.toUByte()))
assert(roundtripOptionRec(null) == null);
