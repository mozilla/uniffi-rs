import uniffi.uniffi_bindgen_tests.*

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
assert(roundtripVecRec(listOf(CollectionsRec(67.toUByte()))) == listOf(CollectionsRec(67.toUByte())))

assert(roundtripHashMap(mapOf("a" to 1u, "b" to 2u)) == mapOf("a" to 1u, "b" to 2u))
assert(roundtripHashSet(setOf("a", "b", "c")) == setOf("a", "b", "c"))
assert(roundtripHashMapU32Key(mapOf(1u to 2u, 2u to 4u)) == mapOf(1u to 2u, 2u to 4u))

assert(
    roundtripRecWithCollections(
        RecWithCollections(
            EnumWithCollections.A(null),
            null,
            listOf(true, false),
            mapOf("a" to 10.toUByte(), "b" to 20.toUByte()),
        )
    ) == RecWithCollections(
        EnumWithCollections.A(null),
        null,
        listOf(true, false),
        mapOf("a" to 10.toUByte(), "b" to 20.toUByte()),
    )
)

assert(roundtripVecHashSet(listOf(setOf("a", "b"))) == listOf(setOf("a", "b")))
assert(roundtripVecHashSet(null) == null)
assert(roundtripComplexCollectionType(listOf(
    mapOf(
        "a" to CollectionsComplexRec(10u, "Test", CollectionsEnum.A(100)),
        "b" to CollectionsComplexRec(20u, "Test2", CollectionsEnum.B(1.0f, true))
    )
)) == listOf(
    mapOf(
        "a" to CollectionsComplexRec(10u, "Test", CollectionsEnum.A(100)),
        "b" to CollectionsComplexRec(20u, "Test2", CollectionsEnum.B(1.0f, true))
    )
))
assert(roundtripComplexCollectionType(null) == null)
