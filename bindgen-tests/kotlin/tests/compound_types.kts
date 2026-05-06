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
assert(roundtripOptionRec(CompoundTypesRec(67.toUByte())) == CompoundTypesRec(67.toUByte()))
assert(roundtripOptionRec(null) == null);

assert(roundtripVecI8(listOf(1.toByte(), 2.toByte(), 3.toByte())) == listOf(1.toByte(), 2.toByte(), 3.toByte()));
assert(roundtripVecU16(listOf(1.toUShort(), 2.toUShort(), 3.toUShort())) == listOf(1.toUShort(), 2.toUShort(), 3.toUShort()));
assert(roundtripVecI16(listOf(1.toShort(), 2.toShort(), 3.toShort())) == listOf(1.toShort(), 2.toShort(), 3.toShort()));
assert(roundtripVecU32(listOf(1u, 2u, 3u)) == listOf(1u, 2u, 3u));
assert(roundtripVecI32(listOf(1, 2, 3)) == listOf(1, 2, 3));
assert(roundtripVecU64(listOf(1uL, 2uL, 3uL)) == listOf(1uL, 2uL, 3uL));
assert(roundtripVecI64(listOf(1L, 2L, 3L)) == listOf(1L, 2L, 3L));
assert(roundtripVecF32(listOf(1.0f, 2.0f, 3.0f)) == listOf(1.0f, 2.0f, 3.0f));
assert(roundtripVecF64(listOf(1.0, 2.0, 3.0)) == listOf(1.0, 2.0, 3.0));
assert(roundtripVecBool(listOf(true, false)) == listOf(true, false));
assert(roundtripVecString(listOf("test-string")) == listOf("test-string"));
assert(roundtripVecRec(listOf(CompoundTypesRec(67.toUByte()))) == listOf(CompoundTypesRec(67.toUByte())))

assert(roundtripHashMap(mapOf("a" to 1u, "b" to 2u)) == mapOf("a" to 1u, "b" to 2u))
assert(roundtripHashSet(setOf("a", "b", "c")) == setOf("a", "b", "c"))
assert(roundtripHashMapU32Key(mapOf(1u to 2u, 2u to 4u)) == mapOf(1u to 2u, 2u to 4u))

assert(
    RecWithCompounds(
        EnumWithCompounds.A(null),
        null,
        listOf(true, false),
        mapOf("a" to 10.toUByte(), "b" to 20.toUByte()),
    ) == RecWithCompounds(
        EnumWithCompounds.A(null),
        null,
        listOf(true, false),
        mapOf("a" to 10.toUByte(), "b" to 20.toUByte()),
    )
)

assert(roundtripComplexCompound(listOf(
    mapOf(
        "a" to CompoundTypesComplexRec(10u, "Test", CompoundTypesEnum.A(100)),
        "b" to CompoundTypesComplexRec(20u, "Test2", CompoundTypesEnum.B(1.0f, true))
    )
)) == listOf(
    mapOf(
        "a" to CompoundTypesComplexRec(10u, "Test", CompoundTypesEnum.A(100)),
        "b" to CompoundTypesComplexRec(20u, "Test2", CompoundTypesEnum.B(1.0f, true))
    )
))
assert(roundtripComplexCompound(null) == null)
assert(roundtripComplexHashSet(listOf(setOf("a", "b"))) == listOf(setOf("a", "b")))
assert(roundtripComplexHashSet(null) == null)
