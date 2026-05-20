import uniffi.uniffi_bindgen_tests.*

assert(roundtripOption(42u)!! == 42u)
assert(roundtripOption(null) == null)
assert(roundtripVec(listOf(1u, 2u, 3u)) == listOf(1u, 2u, 3u))
assert(roundtripHashMap(mapOf("a" to 1u, "b" to 2u)) == mapOf("a" to 1u, "b" to 2u))
assert(roundtripHashSet(setOf("a", "b", "c")) == setOf("a", "b", "c"))
assert(roundtripComplexCompound(listOf(
    mapOf("a" to 1u, "b" to 2u)
)) == listOf(
    mapOf("a" to 1u, "b" to 2u)
))
assert(roundtripComplexCompound(null) == null)
assert(roundtripComplexHashSet(listOf(setOf("a", "b"))) == listOf(setOf("a", "b")))
assert(roundtripComplexHashSet(null) == null)
