import uniffi.trait_methods.*

val m = TraitMethods("yo")
assert(m.toString() == "TraitMethods(yo)")

assert(m == TraitMethods("yo"))
assert(m != TraitMethods("yoyo"))

val map = mapOf(m to 1, TraitMethods("yoyo") to 2)
assert(map[m] == 1)
assert(map[TraitMethods("yoyo")] == 2)

assert(ProcTraitMethods("a") < ProcTraitMethods("b"))
assert(m < TraitMethods("z"))
assert(m <= TraitMethods("z"))
assert(TraitMethods("z") > m)

// Records
assert(UdlRecord(s="yo", i=2) == UdlRecord(s="yo", i=2))
assert(UdlRecord(s="yo", i=2) == UdlRecord(s="yo", i=3))
assert(UdlRecord(s="hi", i=2) != UdlRecord(s="yo", i=3))
assert(UdlRecord(s="a", i=2) < UdlRecord(s="yo", i=1))

assert(TraitRecord(s="yo", i=2).toString() == "TraitRecord { s: \"yo\", i: 2 }")
assert(TraitRecord(s="yo", i=2) == TraitRecord(s="yo", i=2))
assert(TraitRecord(s="yo", i=2) == TraitRecord(s="yo", i=3))
assert(TraitRecord(s="hi", i=2) != TraitRecord(s="yo", i=3))
assert(TraitRecord(s="a", i=2) < TraitRecord(s="yo", i=1))

// Enums

assert((UdlEnum::S)("hello") == (UdlEnum::S)("hello"))
assert((UdlEnum::S)("hello") == (UdlEnum::S)("other"))
assert((UdlEnum::S)("hello") < (UdlEnum::I)(0))
assert((TraitEnum::I)(1) == (TraitEnum::I)(1))
assert((TraitEnum::I)(1) == (TraitEnum::I)(2))
assert((TraitEnum::S)("hello") < (TraitEnum::I)(0))

// nested enums with payloads and Display trait
assert((TraitEnum::S)("hello").toString() == "TraitEnum::S(\"hello\")")
assert((TraitEnum::I)(1).toString() == "TraitEnum::I(1)")
assert((UdlEnum::S)("hello").toString() == "S { s: \"hello\" }")

// flat enum (no payload) with Display
assert(getEnumWithDisplayExport(0u).toString() == "display: One")
assert(getEnumWithDisplayExport(1u).toString() == "display: Two")
assert(getEnumWithDisplayExport(2u).toString() == "display: Three")

// nested enum with another enum (that has Display) as payload
assert(getNestedEnumWithDisplay(0u).toString() == "nested simple: display: One")
assert(getNestedEnumWithDisplay(1u).toString() == "nested complex [test]: display: Two")
assert(getNestedEnumWithDisplay(2u).toString() == "nested simple: display: Three")

// flat enum with Eq/Ord/Hash - uses Kotlin enum class's built-in support
val flatAlpha = getFlatTraitEnum(0u)
assert(flatAlpha == FlatTraitEnum.ALPHA)
assert(getFlatTraitEnum(1u) > flatAlpha)
assert(flatAlpha.toString() == "FlatTraitEnum::flat-alpha")
assert(getFlatTraitEnum(2u).toString() == "FlatTraitEnum::flat-gamma")
val flatSet = setOf(flatAlpha)
assert(flatSet.contains(FlatTraitEnum.ALPHA))
assert(!flatSet.contains(FlatTraitEnum.BETA))

// flat enum with explicit numeric discriminant and Display
val numericRed = getNumericEnum(100u)
assert(numericRed == NumericEnum.RED)
assert(numericRed.value == 100u.toUShort())
assert(numericRed.toString() == "NumericEnum::color-red")
assert(getNumericEnum(200u).toString() == "NumericEnum::color-green")
assert(getNumericEnum(300u).toString() == "NumericEnum::color-blue")
assert(getNumericEnum(999u).value == 300u.toUShort())  // defaults to Blue

// mixed enum with object variants (requires parent-level trait impls - line 117)
// object variants have no fields, so they can't have their own trait impl bodies
// they MUST inherit from the sealed parent's implementations
val objectVariant1 = getMixedEnum(0u) // NoFields - object variant
val objectVariant2 = getMixedEnum(2u) // AnotherNoFields - object variant
val dataVariant1 = getMixedEnum(1u)   // WithString - data class variant
val dataVariant2 = getMixedEnum(3u)   // WithNumber - data class variant

// test toString on object variants - will fail without line 117
assert(objectVariant1.toString() == "MixedEnum::NoFields")
assert(objectVariant2.toString() == "MixedEnum::AnotherNoFields")

// test equals on object variants - will fail without line 117
assert(objectVariant1 == getMixedEnum(0u))
assert(objectVariant2 == getMixedEnum(2u))

// test hashCode on object variants - will fail without line 117
assert(objectVariant1.hashCode() == getMixedEnum(0u).hashCode())
val mixedSet = setOf(objectVariant1, objectVariant2)
assert(mixedSet.contains(getMixedEnum(0u)))

// test compareTo on object variants - will fail without line 117
assert(objectVariant1 < objectVariant2)
assert(objectVariant1 < dataVariant1)
assert(objectVariant2 < dataVariant2)

// test mixed list with object variants - will fail without line 117
val mixedList: List<MixedEnum> = listOf(
    dataVariant2,
    objectVariant2,
    dataVariant1,
    objectVariant1
)
val sortedMixed = mixedList.sorted()
assert(sortedMixed[0] is MixedEnum.NoFields) // object variant first
assert(sortedMixed[3] is MixedEnum.WithNumber) // data variant last

// Errors

// flat error - only toString() is generated (like flat enums)
try {
    throwFlatError(0u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.NotFound) {
    assert(e.toString() == "error: not found")
}

try {
    throwFlatError(1u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.Unauthorized) {
    assert(e.toString() == "error: unauthorized")
}

try {
    throwFlatError(2u)
    throw AssertionError("should have thrown")
} catch (e: FlatException.InternalException) {
    assert(e.toString() == "error: internal error")
}

// error with fields
try {
    throwWithFieldsError(0u)
    throw AssertionError("should have thrown")
} catch (e: WithFieldsException.TooMany) {
    assert(e.toString() == "display: too many items: 100")
}

try {
    throwWithFieldsError(1u)
    throw AssertionError("should have thrown")
} catch (e: WithFieldsException.TooFew) {
    assert(e.toString() == "display: too few items: 0")
}

// nested error with another error (that has Display) as payload
try {
    throwNestedWithDisplayError(0u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Simple) {
    assert(e.toString() == "nested simple error: display: too many items: 42")
}

try {
    throwNestedWithDisplayError(1u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Complex) {
    assert(e.toString() == "nested complex error [nested]: display: too few items: 7")
}

try {
    throwNestedWithDisplayError(2u)
    throw AssertionError("should have thrown")
} catch (e: NestedWithDisplayException.Simple) {
    assert(e.toString() == "nested simple error: display: too few items: 0")
}

// mixed error with flat variants (no fields) and field variants
try {
    throwMixedFieldError(0u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.SimpleFailure) {
    assert(e.toString() == "error: simple failure")
}

try {
    throwMixedFieldError(1u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.ValidationFailed) {
    assert(e.toString() == "error: validation failed with 5 errors")
}

try {
    throwMixedFieldError(2u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.Timeout) {
    assert(e.toString() == "error: timeout")
}

try {
    throwMixedFieldError(3u)
    throw AssertionError("should have thrown")
} catch (e: MixedFieldException.InvalidInput) {
    assert(e.toString() == "error: invalid input: bad data")
}
